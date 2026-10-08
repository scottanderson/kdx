//! Decodes KDX connections in a pcap or pcapng capture.
//!
//! For each TCP connection to the server port, prints both handshakes, finds the
//! account key from the candidate passwords, then decrypts and dumps every session
//! message in capture order.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use kdx_proto::handshake::{compute_auth, Hello, Proof};
use kdx_proto::msg::news::{NewsMessages, NewsgroupEntry};
use kdx_proto::msg::{FilePath, PrivateMessage, ServerInfo, Status, UserRecord};
use kdx_proto::session::{Header, SessionCipher};
use kdx_proto::{AccountKey, MessageType};
use pcap_file::pcap::PcapReader;
use pcap_file::pcapng::{Block, PcapNgReader};
use pcap_file::DataLink;

#[derive(Parser)]
#[command(about = "Decode KDX traffic from a packet capture")]
struct Args {
    /// pcap or pcapng file.
    capture: PathBuf,
    /// Server TCP port.
    #[arg(long, default_value_t = 10700)]
    port: u16,
    /// Candidate account password; repeat for several. The empty password is always tried.
    #[arg(long = "password", short)]
    passwords: Vec<String>,
}

/// One captured frame, numbered from 1 as in Wireshark.
struct Frame {
    number: usize,
    link: DataLink,
    data: Vec<u8>,
}

/// A TCP segment with payload.
struct Segment {
    frame: usize,
    seq: u32,
    payload: Vec<u8>,
}

/// One direction of a TCP connection.
#[derive(Default)]
struct Flow {
    isn: Option<u32>,
    segments: Vec<Segment>,
}

/// A reassembled byte stream and the frame that carried each byte range.
struct Stream {
    data: Vec<u8>,
    /// `(start offset, frame number)`, sorted by offset.
    frames: Vec<(usize, usize)>,
}

impl Stream {
    fn frame_at(&self, offset: usize) -> usize {
        let i = self.frames.partition_point(|&(start, _)| start <= offset);
        self.frames[i.saturating_sub(1)].1
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Dir {
    ClientToServer,
    ServerToClient,
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ClientToServer => "C->S",
            Self::ServerToClient => "S->C",
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let frames = read_frames(&args.capture)?;

    let mut connections: BTreeMap<(SocketAddr, SocketAddr), [Flow; 2]> = BTreeMap::new();
    for frame in &frames {
        let Some((src, dst, tcp)) = parse_tcp(frame) else {
            continue;
        };
        let (key, dir) = if dst.port() == args.port {
            ((src, dst), Dir::ClientToServer)
        } else if src.port() == args.port {
            ((dst, src), Dir::ServerToClient)
        } else {
            continue;
        };
        let flow = &mut connections.entry(key).or_default()[dir as usize];
        if tcp.syn {
            flow.isn = Some(tcp.seq);
        }
        if !tcp.payload.is_empty() {
            flow.segments.push(Segment {
                frame: frame.number,
                seq: tcp.seq,
                payload: tcp.payload,
            });
        }
    }

    let mut keys = vec![("empty password".to_string(), AccountKey::EMPTY_PASSWORD)];
    for (i, password) in args.passwords.iter().enumerate() {
        keys.push((
            format!("--password #{}", i + 1),
            AccountKey::from_password(password.as_bytes()),
        ));
    }

    if connections.is_empty() {
        println!("no TCP connections on port {}", args.port);
    }
    for ((client, server), [c2s, s2c]) in &connections {
        println!("connection {client} -> {server}");
        let streams = [reassemble(c2s), reassemble(s2c)];
        if let Err(err) = dissect(&streams, &keys) {
            println!("  stopped: {err:#}");
        }
        println!();
    }
    Ok(())
}

/// Prints one connection.
fn dissect(streams: &[Stream; 2], keys: &[(String, AccountKey)]) -> Result<()> {
    let dirs = [Dir::ClientToServer, Dir::ServerToClient];
    let mut hellos = Vec::new();
    for (stream, dir) in streams.iter().zip(dirs) {
        let (hello, len) = Hello::decode(&stream.data).with_context(|| format!("{dir} hello"))?;
        println!(
            "  [{}] {dir} hello: service {} format {} version {:#04x} challenge {} nonce {} name {}",
            stream.frame_at(0),
            fourcc(hello.service),
            hello.format,
            hello.version,
            hello.challenge,
            hex::encode(hello.nonce),
            quoted(&hello.name)
        );
        hellos.push((hello, len));
    }

    let mut proofs = Vec::new();
    for ((stream, dir), (_, hello_len)) in streams.iter().zip(dirs).zip(&hellos) {
        let proof =
            Proof::decode(&stream.data[*hello_len..]).with_context(|| format!("{dir} proof"))?;
        println!(
            "  [{}] {dir} proof: status {:?} auth {} response {}",
            stream.frame_at(*hello_len),
            proof.status,
            hex::encode(proof.auth),
            quoted(&proof.response)
        );
        proofs.push(proof);
    }

    let server_nonce = &hellos[1].0.nonce;
    let Some((label, key)) = keys
        .iter()
        .find(|(_, key)| proofs[0].auth == compute_auth(key, server_nonce))
    else {
        bail!("no candidate password matches the client's proof; try --password");
    };
    let server_ok = proofs[1].auth == compute_auth(key, &hellos[0].0.nonce);
    println!(
        "  key: {label} (server proof {})",
        if server_ok {
            "matches"
        } else {
            "DOES NOT match"
        }
    );

    let cipher = SessionCipher::new(key);
    let mut messages = Vec::new();
    for ((stream, dir), (_, hello_len)) in streams.iter().zip(dirs).zip(&hellos) {
        let mut at = hello_len + Proof::LEN;
        while at < stream.data.len() {
            let frame = stream.frame_at(at);
            let Some(block) = stream.data.get(at..at + Header::LEN) else {
                messages.push((
                    frame,
                    dir,
                    Err(format!("{} trailing bytes", stream.data.len() - at)),
                ));
                break;
            };
            let header = match cipher.open_header(block.try_into().unwrap()) {
                Ok(header) => header,
                Err(err) => {
                    messages.push((
                        frame,
                        dir,
                        Err(format!("bad header at stream offset {at}: {err}")),
                    ));
                    break;
                }
            };
            let payload = cipher.open_payload(&header, &stream.data[at + Header::LEN..]);
            match payload {
                Ok(payload) => messages.push((frame, dir, Ok((header, payload)))),
                Err(err) => {
                    messages.push((frame, dir, Err(format!("txid {:08x}: {err}", header.txid))));
                    break;
                }
            }
            at += Header::LEN + header.padded_len();
        }
    }

    messages.sort_by_key(|(frame, dir, _)| (*frame, *dir));
    for (frame, dir, message) in messages {
        match message {
            Ok((header, payload)) => {
                println!(
                    "  [{frame}] {dir} txid {:08x} type {}{} len {}",
                    header.txid,
                    header.kind,
                    if header.is_reply() { " reply" } else { "" },
                    header.len
                );
                match summary(header.kind, &payload) {
                    Ok(Some(text)) => println!("      = {text}"),
                    Ok(None) => {}
                    Err(err) => println!("      ! {err}"),
                }
                print!("{}", hexdump(&payload, "      "));
            }
            Err(err) => println!("  [{frame}] {dir} {err}"),
        }
    }
    Ok(())
}

/// Describes a payload whose layout `kdx_proto::msg` knows.
fn summary(kind: MessageType, payload: &[u8]) -> Result<Option<String>> {
    let user = |u: &UserRecord| {
        format!(
            "user {} {} status {} flags {:#04x} icon {} bytes info {}",
            u.user_id,
            quoted(&u.nick),
            u.status,
            u.flags,
            u.icon.len(),
            quoted(&u.info)
        )
    };
    let text = match kind {
        MessageType::STATUS => match Status::decode(payload)? {
            Status::Success { value } => format!("success {value:#x}"),
            Status::Error {
                code,
                request,
                module,
                line,
                extra,
            } => format!(
                "error {code:#x} for {request} at module {module:#x} line {line} extra {extra:#x}"
            ),
        },
        MessageType::SET_IDENTITY | MessageType::USER_CHANGED | MessageType::USER_JOINED => {
            user(&UserRecord::decode(payload)?.0)
        }
        MessageType::USER_LIST => UserRecord::decode_list(payload)?
            .iter()
            .map(user)
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::SERVER_INFO => {
            let info = ServerInfo::decode(payload)?;
            format!(
                "you are user {}, server {} version {} name {} description {}",
                info.user_id,
                info.server_value,
                info.version,
                quoted(&info.name),
                quoted(&info.description)
            )
        }
        MessageType::SEND_PRIVATE_MESSAGE | MessageType::INCOMING_PRIVATE_MESSAGE => {
            let message = PrivateMessage::decode(payload)?;
            format!("user {} text {}", message.user_id, quoted(&message.text))
        }
        MessageType::GET_FILE_LIST => {
            let path = FilePath::decode(payload)?;
            let names: Vec<String> = path.0.iter().map(|n| quoted(n)).collect();
            format!("path /{}", names.join("/"))
        }
        MessageType::NEWSGROUP_LIST => NewsgroupEntry::decode_list(payload)?
            .iter()
            .map(|g| {
                format!(
                    "group {} {} ({} messages)",
                    g.group_id,
                    quoted(&g.name),
                    g.message_count
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        MessageType::NEWS_MESSAGES => {
            let news = NewsMessages::decode(payload)?;
            let subjects: Vec<String> = news
                .messages
                .iter()
                .map(|m| {
                    format!(
                        "{} {} by {}",
                        m.message_id,
                        quoted(&m.subject),
                        quoted(&m.poster)
                    )
                })
                .collect();
            format!("group {}: {}", news.group_id, subjects.join("; "))
        }
        _ => return Ok(None),
    };
    Ok(Some(text))
}

fn read_frames(path: &PathBuf) -> Result<Vec<Frame>> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let mut frames = Vec::new();
    if bytes.starts_with(&[0x0a, 0x0d, 0x0d, 0x0a]) {
        let mut reader = PcapNgReader::new(&bytes[..])?;
        let mut links = Vec::new();
        while let Some(block) = reader.next_block() {
            let (interface, data) = match block? {
                Block::InterfaceDescription(idb) => {
                    links.push(idb.linktype);
                    continue;
                }
                Block::EnhancedPacket(epb) => (epb.interface_id as usize, epb.data.into_owned()),
                Block::SimplePacket(spb) => (0, spb.data.into_owned()),
                Block::Packet(pb) => (usize::from(pb.interface_id), pb.data.into_owned()),
                _ => continue,
            };
            let link = *links
                .get(interface)
                .context("packet before its interface description")?;
            frames.push(Frame {
                number: frames.len() + 1,
                link,
                data,
            });
        }
    } else {
        let mut reader = PcapReader::new(&bytes[..])?;
        let link = reader.header().datalink;
        while let Some(packet) = reader.next_packet() {
            frames.push(Frame {
                number: frames.len() + 1,
                link,
                data: packet?.data.into_owned(),
            });
        }
    }
    Ok(frames)
}

struct Tcp {
    seq: u32,
    syn: bool,
    payload: Vec<u8>,
}

fn parse_tcp(frame: &Frame) -> Option<(SocketAddr, SocketAddr, Tcp)> {
    let data = &frame.data;
    let packet = match frame.link {
        DataLink::ETHERNET => SlicedPacket::from_ethernet(data),
        DataLink::LINUX_SLL => SlicedPacket::from_linux_sll(data),
        DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data),
        DataLink::NULL | DataLink::LOOP => SlicedPacket::from_ip(data.get(4..)?),
        _ => return None,
    }
    .ok()?;
    let (src, dst): (IpAddr, IpAddr) = match packet.net? {
        NetSlice::Ipv4(ip) => (
            ip.header().source_addr().into(),
            ip.header().destination_addr().into(),
        ),
        NetSlice::Ipv6(ip) => (
            ip.header().source_addr().into(),
            ip.header().destination_addr().into(),
        ),
        _ => return None,
    };
    let TransportSlice::Tcp(tcp) = packet.transport? else {
        return None;
    };
    Some((
        SocketAddr::new(src, tcp.source_port()),
        SocketAddr::new(dst, tcp.destination_port()),
        Tcp {
            seq: tcp.sequence_number(),
            syn: tcp.syn(),
            payload: tcp.payload().to_vec(),
        },
    ))
}

/// Orders segments by sequence number, drops retransmitted bytes, and stops at the first gap.
fn reassemble(flow: &Flow) -> Stream {
    let mut stream = Stream {
        data: Vec::new(),
        frames: Vec::new(),
    };
    let Some(first) = flow.segments.first() else {
        return stream;
    };
    let base = flow.isn.map_or(first.seq, |isn| isn.wrapping_add(1));
    let mut segments: Vec<(usize, &Segment)> = flow
        .segments
        .iter()
        .map(|s| (s.seq.wrapping_sub(base) as usize, s))
        .filter(|&(offset, _)| offset < 1 << 31)
        .collect();
    segments.sort_by_key(|&(offset, s)| (offset, s.frame));
    for (offset, segment) in segments {
        let end = offset + segment.payload.len();
        if offset > stream.data.len() {
            break;
        }
        if end > stream.data.len() {
            stream.frames.push((stream.data.len(), segment.frame));
            stream
                .data
                .extend_from_slice(&segment.payload[stream.data.len() - offset..]);
        }
    }
    stream
}

fn fourcc(value: u32) -> String {
    quoted(&value.to_be_bytes())
}

fn quoted(bytes: &[u8]) -> String {
    format!("\"{}\"", bytes.escape_ascii())
}

fn hexdump(data: &[u8], indent: &str) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        writeln!(
            out,
            "{indent}{:04x}  {:<47}  {ascii}",
            i * 16,
            hex.join(" ")
        )
        .unwrap();
    }
    out
}

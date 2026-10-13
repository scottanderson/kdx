//! Logs in to a KDX server, sends raw requests, and dumps every message received.
//!
//! Each request is `TYPE` or `TYPE:HEX`, for example `605` or `607:00000001`.
//!
//! With `--listen`, it instead waits for one client, plays a GConsole peer
//! (`%HGC`) through the handshake, sends the requests, and dumps what the
//! client sends until it disconnects.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use bytes::Bytes;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use kdx_client::{table, Connection, Login};
use kdx_codec::{Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{Hello, Proof, Status, FORMAT, GCONSOLE_VERSION, SERVICE_GCONSOLE};
use kdx_proto::session::{Header, MessageType, Scramble};
use kdx_proto::{AccountKey, ChallengeTable, Story};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

#[derive(Parser)]
#[command(about = "Send raw requests to a KDX server and dump the replies")]
struct Args {
    /// Server address.
    #[arg(long, default_value = "127.0.0.1:10700")]
    addr: String,
    /// Wait for a client on this address and play a GConsole peer, with GConsole's table.
    #[arg(long, value_name = "ADDR")]
    listen: Option<String>,
    /// Account login; omit for guest.
    #[arg(long, default_value = "")]
    login: String,
    /// Account password; with `--listen`, the password the client must log in with.
    #[arg(long, default_value = "")]
    password: String,
    /// Challenge table, or any KDX program file or archive that contains it; defaults to the cache.
    /// With `--listen`, GConsole's table, or a KDX client or archive.
    #[arg(long)]
    table: Option<PathBuf>,
    /// Nickname sent in the login message.
    #[arg(long, default_value = "probe")]
    nick: String,
    /// Skip the login message (type 100).
    #[arg(long)]
    no_login: bool,
    /// Send the requests on a second, attached connection (as file transfers
    /// do) instead of the login connection.
    #[arg(long)]
    attach: bool,
    /// How long to wait for more messages after each request, in milliseconds.
    #[arg(long, default_value_t = 700)]
    wait: u64,
    /// Requests to send in order: `TYPE` or `TYPE:HEX`.
    requests: Vec<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(addr) = &args.listen {
        return listen(addr, &args).await;
    }
    let table = match &args.table {
        Some(path) => table::load(Story::Kdx, path)?,
        None => table::cached(Story::Kdx).context("no cached KDX story; pass --table")?,
    };
    let login = Login {
        name: args.login.into_bytes(),
        password: args.password.into_bytes(),
    };
    let mut conn = Connection::connect(&args.addr, &login, &table).await?;
    println!(
        "connected; server version {:#04x}",
        conn.server_hello.version
    );

    let mut requests = Vec::new();
    // The login connection must stay open while an attached one is in use.
    let mut _login_conn = None;
    if args.attach {
        let txid = conn
            .send(
                MessageType::SET_IDENTITY,
                &login_record(args.nick.as_bytes()),
            )
            .await?;
        println!(">> txid {txid:08x} type {}", MessageType::SET_IDENTITY);
        let info = loop {
            let (header, payload) = conn.recv().await.context("waiting for server info")?;
            if header.kind == MessageType::SERVER_INFO {
                break payload;
            }
        };
        anyhow::ensure!(info.len() >= 0x28, "server info too short");
        let key: [u8; 32] = info[..32].try_into()?;
        let user_id = u32::from_be_bytes(info[0x24..0x28].try_into()?);
        println!("attaching as user {user_id}");
        let attached = Connection::attach(&args.addr, user_id, key, &table).await?;
        _login_conn = Some(std::mem::replace(&mut conn, attached));
    } else if !args.no_login {
        requests.push((
            MessageType::SET_IDENTITY,
            login_record(args.nick.as_bytes()),
        ));
    }
    requests.extend(parse_requests(&args.requests)?);

    for (kind, payload) in requests {
        let txid = conn.send(kind, &payload).await?;
        print_sent(txid, kind, &payload);
        drain(&mut conn, Duration::from_millis(args.wait)).await?;
    }
    Ok(())
}

/// Prints messages until none arrives for `wait`.
async fn drain(conn: &mut Connection, wait: Duration) -> Result<()> {
    // Receiving is cancel-safe, so a timeout loses no message.
    while let Ok(received) = tokio::time::timeout(wait, conn.recv()).await {
        let (header, payload) = received?;
        print_received(&header, &payload);
    }
    Ok(())
}

/// Accepts one client as a GConsole peer, sends the requests, and prints what it sends.
async fn listen(addr: &str, args: &Args) -> Result<()> {
    let table = match &args.table {
        Some(path) => table::load(Story::Hgc, path)?,
        None => table::cached(Story::Hgc).context("no cached GConsole story; pass --table")?,
    };
    let requests = parse_requests(&args.requests)?;
    let key = AccountKey::from_password(args.password.as_bytes());

    let listener = TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);
    let (stream, peer) = listener.accept().await?;
    println!("connection from {peer}");
    let mut framed = handshake(stream, &key, &table).await?;

    let mut next_txid = random_u32().max(1);
    for (kind, payload) in requests {
        let txid = next_txid;
        next_txid = next_txid.wrapping_add(1).max(1);
        let header = Header {
            txid,
            kind,
            flags: 0,
            len: 0,
        };
        framed
            .send(Frame::Message(header, Bytes::from(payload.clone())))
            .await?;
        print_sent(txid, kind, &payload);
        let wait = Duration::from_millis(args.wait);
        while let Ok(frame) = tokio::time::timeout(wait, framed.next()).await {
            let Some(frame) = frame else {
                println!("closed");
                return Ok(());
            };
            print_frame(frame?)?;
        }
    }
    while let Some(frame) = framed.next().await {
        print_frame(frame?)?;
    }
    println!("closed");
    Ok(())
}

/// Runs the GConsole side of the handshake and switches to the session.
async fn handshake(
    stream: TcpStream,
    key: &AccountKey,
    table: &ChallengeTable,
) -> Result<Framed<TcpStream, SessionCodec>> {
    stream.set_nodelay(true)?;
    let mut framed = Framed::new(stream, HandshakeCodec::new());
    let Handshake::Hello { hello: client, .. } = framed.next().await.context("closed")?? else {
        unreachable!("the codec reads a hello first");
    };
    println!(
        "client hello: service {:#010x} version {:#04x} login {:?}",
        client.service,
        client.version,
        String::from_utf8_lossy(&client.name)
    );
    let ours = Hello {
        service: SERVICE_GCONSOLE,
        format: FORMAT,
        version: GCONSOLE_VERSION,
        challenge: random_u32() as u16 % table.offsets(),
        nonce: random(),
        name: Vec::new(),
    };
    framed
        .send(Handshake::Hello {
            hello: ours.clone(),
            key: random_u32(),
        })
        .await?;
    let Handshake::Proof(proof) = framed.next().await.context("closed")?? else {
        unreachable!("the codec reads a proof after the hello");
    };
    let expected = table.response(ours.challenge).expect("challenge in range");
    if let Err(err) = proof.verify(key, &ours.nonce, &expected) {
        framed
            .send(Handshake::Proof(Proof::new_error(
                &client.nonce,
                Status::Other(1),
            )))
            .await?;
        bail!("client proof: {err}");
    }
    let response = table
        .response(client.challenge)
        .context("client challenge out of range")?;
    framed
        .send(Handshake::Proof(Proof::new_ok(
            key,
            &client.nonce,
            response,
        )))
        .await?;
    println!("logged in");
    Ok(framed.map_codec(|_| SessionCodec::server(key).with_scramble(Scramble::GConsole)))
}

fn print_frame(frame: Frame) -> Result<()> {
    match frame {
        Frame::Message(header, payload) => print_received(&header, &payload),
        Frame::DataHeader(header) => bail!("unexpected {} data stream", header.kind),
        Frame::Data(_) => bail!("unexpected transfer data"),
    }
    Ok(())
}

fn print_sent(txid: u32, kind: MessageType, payload: &[u8]) {
    println!(">> txid {txid:08x} type {} len {}", kind, payload.len());
    print!("{}", hexdump(payload));
}

fn print_received(header: &Header, payload: &[u8]) {
    let reply = if header.is_reply() { " reply" } else { "" };
    println!(
        "<< txid {:08x} type {}{reply} len {}",
        header.txid, header.kind, header.len
    );
    print!("{}", hexdump(payload));
}

fn parse_requests(requests: &[String]) -> Result<Vec<(MessageType, Vec<u8>)>> {
    requests
        .iter()
        .map(|request| {
            let (kind, hex) = request.split_once(':').unwrap_or((request, ""));
            let kind: u32 = kind
                .parse()
                .with_context(|| format!("bad type in {request}"))?;
            let payload = decode_hex(hex).with_context(|| format!("bad hex in {request}"))?;
            Ok((MessageType(kind), payload))
        })
        .collect()
}

fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).expect("system random source");
    bytes
}

fn random_u32() -> u32 {
    u32::from_be_bytes(random())
}

/// A login (type 100) user record with no icon and the given nickname.
fn login_record(nick: &[u8]) -> Vec<u8> {
    let mut record = vec![0; 0x1c];
    record[0x19] = nick.len() as u8;
    record.extend_from_slice(nick);
    record
}

fn decode_hex(hex: &str) -> Result<Vec<u8>> {
    let hex: String = hex
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect();
    anyhow::ensure!(hex.len().is_multiple_of(2), "odd number of hex digits");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).context("invalid hex"))
        .collect()
}

fn hexdump(data: &[u8]) -> String {
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
        out.push_str(&format!(
            "   {:04x}  {:<47}  {ascii}\n",
            i * 16,
            hex.join(" ")
        ));
    }
    out
}

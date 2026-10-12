//! Runs every capture fixture in `kdx-proto/tests/fixtures/` through the codecs.

use std::path::{Path, PathBuf};

use bytes::{Bytes, BytesMut};
use kdx_codec::{Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{Hello, Proof};
use kdx_proto::msg::ServerInfo;
use kdx_proto::session::{
    padding_fill, Header, MessageType, SessionCipher, CLIENT_MAX_PAYLOAD_LEN,
    SERVER_MAX_PAYLOAD_LEN,
};
use kdx_proto::AccountKey;
use tokio_util::codec::{Decoder, Encoder};

/// The password of the lab's admin account.
const PASSWORD: &[u8] = b"lab-only-pw!1";

/// Piece sizes to feed the decoders, so frames split at every kind of boundary.
const PIECES: [usize; 5] = [1, 7, 16, 1500, usize::MAX];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../kdx-proto/tests/fixtures")
}

fn read(name: &str, dir: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(format!("{name}.{dir}.bin"))).unwrap()
}

/// Every connection, named like `probe-32980`.
fn connections() -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(fixtures())
        .unwrap()
        .filter_map(|entry| {
            let name = entry.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".c2s.bin").map(str::to_owned)
        })
        .collect();
    names.sort();
    assert!(names.len() >= 25, "only {} fixtures", names.len());
    names
}

/// The key of a connection, as `kdx-proto/tests/fixtures/README.md` gives it.
fn key(name: &str) -> AccountKey {
    let parent = match name {
        "kdx-news-40434" | "linux-accounts-news-36078" => return AccountKey::EMPTY_PASSWORD,
        "kdx-accounts-58336" => return AccountKey::from_password(b"rs-test-pw"),
        "probe-33082" => "probe-33068",
        "probe-48796" => "probe-48784",
        "probe-53518" => "probe-53514",
        "probe-56352" => "probe-56346",
        "linux-files-50904" | "linux-files-51076" | "linux-files-54196" => "linux-files-59246",
        _ => return AccountKey::from_password(PASSWORD),
    };
    let (_, frames) = decode(
        &read(parent, "s2c"),
        &key(parent),
        CLIENT_MAX_PAYLOAD_LEN,
        16,
    );
    let info = frames
        .iter()
        .find_map(|frame| match frame {
            Frame::Message(h, p) if h.kind == MessageType::SERVER_INFO => Some(p),
            _ => None,
        })
        .unwrap();
    AccountKey(ServerInfo::decode(info).unwrap().session_key)
}

/// Feeds `stream` to the codecs in pieces of `piece` bytes, switching codecs after the proof.
fn decode(
    stream: &[u8],
    key: &AccountKey,
    max_len: u32,
    piece: usize,
) -> (Vec<Handshake>, Vec<Frame>) {
    let mut handshake = HandshakeCodec::new();
    let mut session = SessionCodec::new(key, max_len);
    let (mut greeting, mut frames) = (Vec::new(), Vec::new());
    let mut buf = BytesMut::new();
    for chunk in stream.chunks(piece.min(stream.len())) {
        buf.extend_from_slice(chunk);
        while !handshake.is_done() {
            match handshake.decode(&mut buf).unwrap() {
                Some(item) => greeting.push(item),
                None => break,
            }
        }
        if handshake.is_done() {
            while let Some(frame) = session.decode(&mut buf).unwrap() {
                frames.push(frame);
            }
        }
    }
    assert!(session.decode_eof(&mut buf).unwrap().is_none());
    assert!(buf.is_empty());
    (greeting, frames)
}

/// Decrypts the messages after the hello and proof with `kdx-proto` alone, as the proto tests do.
fn reference(stream: &[u8], key: &AccountKey) -> Vec<(Header, Vec<u8>)> {
    let cipher = SessionCipher::new(key);
    let (_, hello_len) = Hello::decode(stream).unwrap();
    let mut at = hello_len + Proof::LEN;
    let mut out = Vec::new();
    while at < stream.len() {
        let header = cipher
            .open_data_header(stream[at..at + 16].try_into().unwrap())
            .unwrap();
        at += Header::LEN;
        out.push((header, cipher.open_payload(&header, &stream[at..]).unwrap()));
        at += header.padded_len();
    }
    out
}

/// Joins each data stream into one message, to compare against [`reference`].
fn join(frames: &[Frame]) -> Vec<(Header, Vec<u8>)> {
    let mut out: Vec<(Header, Vec<u8>)> = Vec::new();
    for frame in frames {
        match frame {
            Frame::Message(h, p) => out.push((*h, p.to_vec())),
            Frame::DataHeader(h) => out.push((*h, Vec::new())),
            Frame::Data(d) => out.last_mut().unwrap().1.extend_from_slice(d),
        }
    }
    out
}

fn encode(greeting: Vec<Handshake>, frames: Vec<Frame>, key: &AccountKey) -> Vec<u8> {
    let mut out = BytesMut::new();
    let mut handshake = HandshakeCodec::new();
    for item in greeting {
        handshake.encode(item, &mut out).unwrap();
    }
    let mut session = SessionCodec::client(key);
    for frame in frames {
        session.encode(frame, &mut out).unwrap();
    }
    assert!(session.is_idle());
    out.to_vec()
}

#[test]
fn decode_matches_kdx_proto() {
    for name in connections() {
        let key = key(&name);
        for (dir, max_len) in [
            ("c2s", SERVER_MAX_PAYLOAD_LEN),
            ("s2c", CLIENT_MAX_PAYLOAD_LEN),
        ] {
            let stream = read(&name, dir);
            let expected = reference(&stream, &key);
            for piece in PIECES {
                let (greeting, frames) = decode(&stream, &key, max_len, piece);
                assert!(
                    matches!(
                        greeting.as_slice(),
                        [Handshake::Hello { .. }, Handshake::Proof(_)]
                    ),
                    "{name}.{dir}"
                );
                assert_eq!(join(&frames), expected, "{name}.{dir} in pieces of {piece}");
            }
        }
    }
}

#[test]
fn reencode_byte_for_byte() {
    for name in connections() {
        let key = key(&name);
        for (dir, max_len) in [
            ("c2s", SERVER_MAX_PAYLOAD_LEN),
            ("s2c", CLIENT_MAX_PAYLOAD_LEN),
        ] {
            let stream = read(&name, dir);
            let expected = without_download_padding(&stream, &key);
            for piece in [7, usize::MAX] {
                let (greeting, frames) = decode(&stream, &key, max_len, piece);
                let encoded = encode(greeting, frames, &key);
                assert!(
                    without_download_padding(&encoded, &key) == expected,
                    "{name}.{dir} in pieces of {piece}"
                );
            }
        }
    }
}

/// Zeroes the last block of 414 data that ends mid-block, whose padding the encoder can't reproduce.
fn without_download_padding(stream: &[u8], key: &AccountKey) -> Vec<u8> {
    let cipher = SessionCipher::new(key);
    let mut out = stream.to_vec();
    let (_, hello_len) = Hello::decode(stream).unwrap();
    let mut at = hello_len + Proof::LEN;
    while at < stream.len() {
        let header = cipher
            .open_data_header(stream[at..at + 16].try_into().unwrap())
            .unwrap();
        at += Header::LEN + header.padded_len();
        if header.kind == MessageType::DOWNLOAD_FILE_DATA && !header.len.is_multiple_of(16) {
            out[at - 16..at].fill(0);
        }
    }
    out
}

#[test]
fn transfer_padding() {
    // Server 1.620 pads 414 data with some other value; Client 1.620 pads 417 data with the fill.
    for (name, dir, kind, padded_with_fill) in [
        (
            "linux-files-51076",
            "s2c",
            MessageType::DOWNLOAD_FILE_DATA,
            false,
        ),
        (
            "linux-files-54196",
            "s2c",
            MessageType::DOWNLOAD_FILE_DATA,
            false,
        ),
        (
            "linux-files-50904",
            "c2s",
            MessageType::UPLOAD_FILE_DATA,
            true,
        ),
    ] {
        let key = key(name);
        let stream = read(name, dir);
        let cipher = SessionCipher::new(&key);
        let (_, hello_len) = Hello::decode(&stream).unwrap();
        let mut at = hello_len + Proof::LEN;
        let mut seen = false;
        while at < stream.len() {
            let header = cipher
                .open_data_header(stream[at..at + 16].try_into().unwrap())
                .unwrap();
            at += Header::LEN;
            let mut data = stream[at..at + header.padded_len()].to_vec();
            at += header.padded_len();
            if header.kind != kind || header.len.is_multiple_of(16) {
                continue;
            }
            cipher.decrypt_blocks(&mut data);
            let fill = padding_fill(header.txid);
            let pad = &data[header.len as usize..];
            let filled = pad
                .iter()
                .enumerate()
                .all(|(i, &b)| b == fill[(header.len as usize + i) % 4]);
            assert_eq!(filled, padded_with_fill, "{name}.{dir}");
            seen = true;
        }
        assert!(seen, "no padded {kind} in {name}.{dir}");
    }
}

#[test]
fn downloads_stream_in_chunks() {
    let key = key("probe-33082");
    let (_, frames) = decode(
        &read("probe-33082", "s2c"),
        &key,
        CLIENT_MAX_PAYLOAD_LEN,
        usize::MAX,
    );
    let data: Vec<&Bytes> = frames
        .iter()
        .filter_map(|frame| match frame {
            Frame::Data(d) => Some(d),
            _ => None,
        })
        .collect();
    // 70000 bytes come as one 64 KiB chunk and the rest.
    assert_eq!(
        data.iter().map(|d| d.len()).collect::<Vec<_>>(),
        [0x10000, 70000 - 0x10000]
    );
    let headers: Vec<Header> = frames
        .iter()
        .filter_map(|frame| match frame {
            Frame::DataHeader(h) => Some(*h),
            _ => None,
        })
        .collect();
    assert_eq!(headers.len(), 2);
    assert!(headers
        .iter()
        .all(|h| h.kind == MessageType::DOWNLOAD_FILE_DATA));
    assert_eq!((headers[0].len, headers[1].len), (70000, 0));
}

#[test]
fn upload_streams() {
    let key = key("probe-48796");
    let (_, frames) = decode(
        &read("probe-48796", "c2s"),
        &key,
        SERVER_MAX_PAYLOAD_LEN,
        usize::MAX,
    );
    let at = frames
        .iter()
        .position(|frame| matches!(frame, Frame::DataHeader(h) if h.kind == MessageType::UPLOAD_FILE_DATA))
        .unwrap();
    let expected: Vec<u8> = (0..100).collect();
    assert!(matches!(&frames[at + 1], Frame::Data(d) if d[..] == expected[..]));
    assert!(matches!(&frames[at + 2], Frame::DataHeader(h) if h.len == 0));
}

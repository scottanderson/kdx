//! Checks against `captures/linux-gconsole.pcapng`: Client 1.620 for Linux as
//! `admin`, connected to `kdx-probe --listen` as a GConsole peer.

use std::path::PathBuf;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use kdx_proto::handshake::{Hello, Proof, GCONSOLE_VERSION, SERVICE_GCONSOLE};
use kdx_proto::msg::gconsole::{Color, Draw, DrawRecord, Key, Mouse, OpenWindow, Point, Rect};
use kdx_proto::session::{Header, MessageType, Scramble, SessionCipher};
use kdx_proto::{AccountKey, ChallengeTable, GConsoleTable};

mod common;

const C2S: &[u8] = include_bytes!("fixtures/linux-gconsole-51924.c2s.bin");
const S2C: &[u8] = include_bytes!("fixtures/linux-gconsole-51924.s2c.bin");

fn cipher() -> SessionCipher {
    SessionCipher::new(&AccountKey::from_password(common::PASSWORD))
        .with_scramble(Scramble::GConsole)
}

fn of_type(messages: &[(Header, Vec<u8>)], kind: MessageType) -> Vec<&[u8]> {
    messages
        .iter()
        .filter(|(h, _)| h.kind == kind)
        .map(|(_, p)| p.as_slice())
        .collect()
}

#[test]
fn challenge_responses() {
    // The table is not in the repository; skip unless the environment, lab binaries, or `kdx table fetch` provide it.
    let Some(table) = env_table().or_else(vendor_table).or_else(cached_table) else {
        let missing = "no GConsole table in HGC_CHALLENGE_TABLE, vendor/haxial, or the kdx cache";
        // CI sets KDX_REQUIRE_TABLE so a missing table fails instead of skipping.
        assert!(std::env::var_os("KDX_REQUIRE_TABLE").is_none(), "{missing}");
        eprintln!("skipping: {missing}");
        return;
    };
    let (client, client_len) = Hello::decode(C2S).unwrap();
    let (peer, peer_len) = Hello::decode(S2C).unwrap();
    assert_eq!(
        (peer.service, peer.version),
        (SERVICE_GCONSOLE, GCONSOLE_VERSION)
    );
    // Frame 6: the peer's offset is past the end of the KDX table, so the client answered from GConsole's.
    assert!(peer.challenge >= ChallengeTable::OFFSETS);
    let client_proof = Proof::decode(&C2S[client_len..]).unwrap();
    let peer_proof = Proof::decode(&S2C[peer_len..]).unwrap();
    assert_eq!(table.response(peer.challenge), Some(client_proof.response));
    assert_eq!(table.response(client.challenge), Some(peer_proof.response));
}

/// The table in `HGC_CHALLENGE_TABLE`, as base64, which CI sets from a secret.
fn env_table() -> Option<GConsoleTable> {
    let b64 = std::env::var("HGC_CHALLENGE_TABLE")
        .ok()
        .filter(|v| !v.trim().is_empty())?;
    let bytes = STANDARD
        .decode(b64.trim())
        .expect("HGC_CHALLENGE_TABLE is not base64");
    Some(GConsoleTable::from_bytes(&bytes).unwrap())
}

/// The table extracted from the client binary that `vendor/fetch` downloads.
fn vendor_table() -> Option<GConsoleTable> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../vendor/haxial/KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe"
    );
    Some(GConsoleTable::extract(&std::fs::read(path).ok()?).unwrap())
}

/// The table `kdx table fetch` saved, at the path `kdx_client::table::cache_path` uses.
fn cached_table() -> Option<GConsoleTable> {
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    let bytes = std::fs::read(dir.join("kdx/table/hgc.bin")).ok()?;
    Some(GConsoleTable::from_bytes(&bytes).unwrap())
}

#[test]
fn open_window() {
    let sent = common::messages(S2C, &cipher());
    // Frame 10.
    let wire = of_type(&sent, MessageType::GCONSOLE_OPEN_WINDOW)[0];
    let open = OpenWindow::decode(wire).unwrap();
    assert_eq!((open.width, open.height), (0x200, 0x180));
    assert_eq!(open.color, Color(0x0033_6699));
    assert_eq!(open.encode(), wire);
}

#[test]
fn draw() {
    let sent = common::messages(S2C, &cipher());
    // Frame 14: a record of each kind.
    let wire = of_type(&sent, MessageType::GCONSOLE_DRAW)[0];
    let draw = Draw::decode(wire).unwrap();
    let kinds: Vec<u16> = draw.0.iter().map(DrawRecord::kind).collect();
    assert_eq!(
        kinds,
        [
            2, 100, 100, 100, 101, 108, 102, 102, 102, 107, 103, 104, 105, 106, 109, 2, 100, 4, 2,
            100, 3
        ]
    );
    assert!(!draw.0.iter().any(|r| matches!(r, DrawRecord::Other { .. })));
    assert_eq!(
        draw.0[9],
        DrawRecord::Polygon {
            id: 1,
            value_02: 0,
            color: Color(0x0000_ff00),
            origin: Point { x: 0, y: 0 },
            points: vec![
                Point { x: 0x110, y: 0xa0 },
                Point { x: 0x130, y: 0x60 },
                Point { x: 0x150, y: 0xa0 },
            ],
        }
    );
    assert_eq!(
        draw.0[17],
        DrawRecord::Move {
            id: 2,
            to: Point { x: 0x180, y: 0x120 },
        }
    );
    let DrawRecord::Create {
        rect,
        id,
        flags,
        object_type,
        ..
    } = draw.0[0]
    else {
        panic!("{:?}", draw.0[0]);
    };
    assert_eq!(
        rect,
        Rect {
            left: 0,
            top: 0,
            right: 0x200,
            bottom: 0x180
        }
    );
    assert_eq!((id, flags, object_type), (1, 8, 1));
    assert_eq!(draw.encode().unwrap(), wire);
}

#[test]
fn keys() {
    let received = common::messages(C2S, &cipher());
    let keys: Vec<_> = of_type(&received, MessageType::GCONSOLE_KEY);
    for wire in &keys {
        assert_eq!(Key::decode(wire).unwrap().encode(), **wire);
    }
    // Frame 48: `a` pressed.
    let a = Key::decode(keys[3]).unwrap();
    assert_eq!((a.kind, a.usage, a.character, a.modifiers), (1, 4, 0x61, 0));
    // Frame 54: Shift+A, with Shift held.
    let shift_a = Key::decode(keys[6]).unwrap();
    assert_eq!(
        (
            shift_a.kind,
            shift_a.usage,
            shift_a.character,
            shift_a.modifiers
        ),
        (1, 4, 0x41, 8)
    );
    // Frame 50: a release carries no usage or character.
    let up = Key::decode(keys[4]).unwrap();
    assert_eq!((up.kind, up.usage, up.character), (2, 0, 0));
}

#[test]
fn mouse() {
    let received = common::messages(C2S, &cipher());
    let mice: Vec<Mouse> = of_type(&received, MessageType::GCONSOLE_MOUSE)
        .into_iter()
        .map(|wire| {
            let mouse = Mouse::decode(wire).unwrap();
            assert_eq!(mouse.encode(), wire);
            mouse
        })
        .collect();
    let events: Vec<_> = mice
        .iter()
        .map(|m| (m.button, m.kind, m.x, m.y, m.wheel))
        .collect();
    assert_eq!(
        events,
        [
            // Frames 20–26: left clicks.
            (1, 5, 100, 50, 0),
            (1, 6, 100, 50, 0),
            (1, 5, 150, 100, 0),
            (1, 6, 150, 100, 0),
            // Frames 28–30: a drag, with no message for the moves.
            (1, 5, 200, 150, 0),
            (1, 6, 300, 200, 0),
            // Frames 32–38: right and middle clicks.
            (2, 5, 100, 200, 0),
            (2, 6, 100, 200, 0),
            (3, 5, 100, 200, 0),
            (3, 6, 100, 200, 0),
            // Frames 40–46: two messages for each wheel step.
            (4, 10, 100, 200, 120),
            (4, 10, 100, 200, 120),
            (5, 10, 100, 200, -120),
            (5, 10, 100, 200, -120),
        ]
    );
    assert_eq!(mice[6].value_04, 0x0004_0000);
}

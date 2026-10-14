//! Replays requests from `captures/linux-tracker.pcapng` and compares the replies with Tracker 1.110's.

use std::net::SocketAddr;

use kdx_proto::handshake::{Hello, Proof};
use kdx_proto::msg::tracker::ServerListEntry;
use kdx_proto::session::{Header, MessageType, SessionCipher};
use kdx_proto::AccountKey;

use crate::{error, status, Reply, Settings, State};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../kdx-proto/tests/fixtures");
const ADMIN_PASSWORD: &[u8] = b"lab-tracker-pw!1";

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("{FIXTURES}/{name}")).unwrap()
}

/// Every message after the handshake of one direction of a fixture connection.
fn messages(name: &str, key: &AccountKey) -> Vec<(Header, Vec<u8>)> {
    let stream = fixture(name);
    let cipher = SessionCipher::new(key);
    let (_, hello_len) = Hello::decode(&stream).unwrap();
    let mut at = hello_len + Proof::LEN;
    let mut out = Vec::new();
    while at < stream.len() {
        let block = stream[at..at + Header::LEN].try_into().unwrap();
        let header = cipher.open_data_header(block).unwrap();
        at += Header::LEN;
        out.push((header, cipher.open_payload(&header, &stream[at..]).unwrap()));
        at += header.padded_len() as usize;
    }
    out
}

/// One connection's requests and the tracker's replies, by txid.
struct Session {
    requests: Vec<(Header, Vec<u8>)>,
    replies: Vec<(Header, Vec<u8>)>,
}

impl Session {
    fn new(port: u16, key: &AccountKey) -> Self {
        Self {
            requests: messages(&format!("linux-tracker-{port}.c2s.bin"), key),
            replies: messages(&format!("linux-tracker-{port}.s2c.bin"), key),
        }
    }

    /// The `n`th request of type `kind` and the reply to it.
    fn exchange(&self, kind: MessageType, n: usize) -> (&[u8], Reply) {
        let (header, payload) = self
            .requests
            .iter()
            .filter(|(h, _)| h.kind == kind)
            .nth(n)
            .unwrap();
        let (reply, body) = self
            .replies
            .iter()
            .find(|(h, _)| h.txid == header.txid && h.is_reply())
            .unwrap();
        let reply = match reply.kind {
            MessageType::STATUS => Reply::Error(u32::from_be_bytes(body[..4].try_into().unwrap())),
            kind => Reply::Message(kind, body.clone()),
        };
        (payload, reply)
    }
}

fn peer() -> SocketAddr {
    "127.0.0.1:40000".parse().unwrap()
}

/// A tracker in the state of the capture's at its start.
fn fresh() -> State {
    State::new(Settings::new(b"admin", ADMIN_PASSWORD))
}

fn reply(state: &mut State, kind: MessageType, payload: &[u8], admin: bool) -> Reply {
    state.handle(kind, payload, admin, peer())
}

fn register(state: &mut State, frame: u32) {
    let datagram = fixture(&format!("linux-tracker-reg-{frame}.bin"));
    state
        .receive("127.0.0.1:50000".parse().unwrap(), &datagram)
        .unwrap();
}

/// Replays the `n`th request of `kind` in `session` and checks the reply.
///
/// First-seen times in 703 are the capture's, which this tracker can't reproduce, so they are copied.
fn replay(state: &mut State, session: &Session, kind: MessageType, n: usize, admin: bool) {
    let (request, expected) = session.exchange(kind, n);
    let mut ours = state.handle(kind, request, admin, peer());
    if let (Reply::Message(MessageType::TRACKER_SERVER_LIST, ours), Reply::Message(_, theirs)) =
        (&mut ours, &expected)
    {
        let theirs = ServerListEntry::decode_list(theirs).unwrap();
        let mut list = ServerListEntry::decode_list(ours).unwrap();
        for (ours, theirs) in list.records.iter_mut().zip(&theirs.records) {
            ours.first_seen = theirs.first_seen;
        }
        *ours = ServerListEntry::encode_list(&list).unwrap();
    }
    assert_eq!(ours, expected, "{kind} #{n}");
}

#[test]
fn guest_browses_a_fresh_tracker() {
    let guest = Session::new(60698, &AccountKey::EMPTY_PASSWORD);
    let mut state = fresh();
    register(&mut state, 23);

    // Frames 33/34: eight groups, the registration counted in General.
    replay(
        &mut state,
        &guest,
        MessageType::GET_TRACKER_GROUP_LIST,
        0,
        false,
    );
    // Frames 36/37 list General; 57/58 list Chat, which is empty.
    replay(
        &mut state,
        &guest,
        MessageType::GET_TRACKER_SERVER_LIST,
        0,
        false,
    );
    replay(
        &mut state,
        &guest,
        MessageType::GET_TRACKER_SERVER_LIST,
        1,
        false,
    );
    // Frames 60/61: a guest may not read the settings.
    replay(
        &mut state,
        &guest,
        MessageType::GET_TRACKERSERV_SETTINGS,
        0,
        false,
    );
    let (_, body) = guest
        .replies
        .iter()
        .find(|(h, _)| h.kind == MessageType::STATUS)
        .unwrap();
    assert_eq!(
        status(error::ACCESS_DENIED, MessageType::GET_TRACKERSERV_SETTINGS),
        *body
    );
}

#[test]
fn admin_reads_and_changes_the_settings() {
    let key = AccountKey::from_password(ADMIN_PASSWORD);
    let admin = Session::new(41900, &key);
    let group_edit = Session::new(34704, &key);
    let mut state = fresh();
    register(&mut state, 23);

    // Frames 79/80, then 712 at 82, and the 711 at 237 that shows its effect.
    replay(
        &mut state,
        &admin,
        MessageType::GET_TRACKERSERV_SETTINGS,
        0,
        true,
    );
    replay(
        &mut state,
        &admin,
        MessageType::SET_TRACKERSERV_SETTINGS,
        0,
        true,
    );
    replay(
        &mut state,
        &group_edit,
        MessageType::GET_TRACKER_GROUP_LIST,
        0,
        true,
    );
    replay(
        &mut state,
        &group_edit,
        MessageType::GET_TRACKERSERV_SETTINGS,
        0,
        true,
    );
    assert_eq!(state.settings.name, b"Lab Tracker");
    // The client can't change the admin password: 712 leaves the admin section unmarked.
    assert_eq!(state.settings.admin_key, key);
    // Frame 232: the greeting the tracker sends after login.
    let (_, greeting) = group_edit
        .replies
        .iter()
        .find(|(h, _)| h.kind == MessageType::SERVER_GREETING)
        .unwrap();
    assert_eq!(state.settings.greeting_text(), Some(&greeting[..]));

    // Frames 240/241 add the group Lab, which drops the registrations: General is empty at 244.
    replay(
        &mut state,
        &group_edit,
        MessageType::SET_TRACKERSERV_SETTINGS,
        0,
        true,
    );
    replay(
        &mut state,
        &group_edit,
        MessageType::GET_TRACKER_SERVER_LIST,
        0,
        true,
    );
    // Frame 247 lists nine groups, and Lab is empty at 250 until registration 255 asks for it.
    replay(
        &mut state,
        &group_edit,
        MessageType::GET_TRACKER_GROUP_LIST,
        1,
        true,
    );
    replay(
        &mut state,
        &group_edit,
        MessageType::GET_TRACKER_SERVER_LIST,
        1,
        true,
    );
    register(&mut state, 255);
    replay(
        &mut state,
        &group_edit,
        MessageType::GET_TRACKER_SERVER_LIST,
        2,
        true,
    );

    // Frames 85/86 hold counts from a different run, so check only that 721 is admin-only.
    assert_eq!(
        reply(
            &mut state,
            MessageType::GET_TRACKERSERV_STATISTICS,
            &[],
            false
        ),
        Reply::Error(error::ACCESS_DENIED)
    );
}

#[test]
fn bad_server_list_requests() {
    let mut state = fresh();
    let kind = MessageType::GET_TRACKER_SERVER_LIST;
    assert_eq!(
        reply(&mut state, kind, &[0, 0], false),
        Reply::Error(error::SHORT)
    );
    assert_eq!(
        reply(&mut state, kind, &[0, 0, 5, 0, b'G'], false),
        Reply::Error(error::SHORT)
    );
    assert_eq!(
        reply(&mut state, kind, &[0, 0, 1, 0, b'G'], false),
        Reply::Error(error::NOT_FOUND)
    );
    assert_eq!(reply(&mut state, MessageType(999), &[], false), Reply::None);
}

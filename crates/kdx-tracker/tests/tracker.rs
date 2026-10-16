//! Runs the tracker on loopback and browses it with `kdx-client`.

use std::net::SocketAddr;
use std::time::Duration;

use kdx_client::{Connection, Login};
use kdx_proto::handshake::TRACKER_SUB_VERSION;
use kdx_proto::msg::tracker::{GetServerList, GroupListEntry, ServerListEntry, TrackerStats};
use kdx_proto::session::MessageType;
use kdx_proto::Story;
use kdx_tracker::{Settings, Sockets, Tracker, MAX_CONNECTIONS_PER_IP};
use tokio::net::UdpSocket;

const PASSWORD: &[u8] = b"lab-tracker-pw!1";
/// Server 1.620's first registration in `captures/linux-tracker.pcapng`, frame 23.
const REGISTRATION: &[u8] =
    include_bytes!("../../kdx-proto/tests/fixtures/linux-tracker-reg-23.bin");

#[tokio::test(flavor = "multi_thread")]
async fn browse() {
    let table = kdx_test_tables::table(Story::Kdx);
    let sockets = Sockets::bind("127.0.0.1".parse().unwrap(), 0)
        .await
        .unwrap();
    let addr = sockets.local_addr().unwrap();
    let settings = Settings::new(b"admin", PASSWORD);
    tokio::spawn(Tracker::new(settings, None, table.clone()).run(sockets));

    let udp = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    udp.send_to(REGISTRATION, addr).await.unwrap();

    let guest = Login::default();
    let mut conn = Connection::connect(addr, &guest, &table).await.unwrap();
    assert_eq!(conn.server_hello.sub_version, TRACKER_SUB_VERSION);
    let general = loop {
        let groups = GroupListEntry::decode_list(&request(&mut conn, 700, &[]).await).unwrap();
        let general = groups.into_iter().find(|g| g.name == b"General").unwrap();
        if general.servers > 0 {
            break general;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    assert_eq!(general.servers, 1);
    let list = GetServerList {
        group: b"General".to_vec(),
        ..GetServerList::default()
    };
    let servers =
        ServerListEntry::decode_list(&request(&mut conn, 702, &list.encode().unwrap()).await)
            .unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, b"Lab Server");
    assert_eq!(servers[0].socket_addr(), "127.0.0.1:10700".parse().unwrap());

    let me = request(&mut conn, 115, &[]).await;
    assert_eq!(usize::from(me[0]), me.len() - 1);
    let me: SocketAddr = std::str::from_utf8(&me[1..]).unwrap().parse().unwrap();
    assert_eq!(me.ip(), addr.ip());
    let denied = request(&mut conn, 720, &[]).await;
    assert_eq!(denied, [0, 1, 0, 0x14, 0, 0, 0x02, 0xd0]);

    let admin = Login {
        name: b"ADMIN".to_vec(),
        password: PASSWORD.to_vec(),
    };
    let mut conn = Connection::connect(addr, &admin, &table).await.unwrap();
    let stats = TrackerStats::decode(&request(&mut conn, 720, &[]).await).unwrap();
    assert_eq!((stats.connections, stats.servers), (2, 1));

    let wrong = Login {
        name: b"admin".to_vec(),
        password: b"wrong".to_vec(),
    };
    assert!(Connection::connect(addr, &wrong, &table).await.is_err());

    // Two connections are open; the limit refuses the sixth.
    let mut extra = Vec::new();
    for _ in 2..MAX_CONNECTIONS_PER_IP {
        extra.push(Connection::connect(addr, &guest, &table).await.unwrap());
    }
    assert!(Connection::connect(addr, &guest, &table).await.is_err());
}

/// Sends a request and returns the payload of its reply.
async fn request(conn: &mut Connection, kind: u32, payload: &[u8]) -> Vec<u8> {
    let txid = conn.send(MessageType(kind), payload).await.unwrap();
    let (header, payload) = conn.recv().await.unwrap();
    assert_eq!(header.txid, txid);
    assert!(header.is_reply());
    payload
}

//! The client against a fake server built on `kdx-codec`, with made-up challenge responses.

use std::io::Cursor;

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use kdx_codec::{Frame, Handshake, HandshakeCodec, SessionCodec};
use kdx_proto::handshake::{Hello, Proof, SERVER_SUB_VERSION, SUB_PROTOCOL_KDX, VERSION};
use kdx_proto::msg::tracker::{GetServerList, GroupListEntry};
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, UploadFile, UploadReply};
use kdx_proto::msg::{FilePath, ServerInfo, Status, UserRecord};
use kdx_proto::session::{Header, MessageType};
use kdx_proto::AccountKey;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use crate::{fake_response, Client, Login, Responses, ServerError, Tracker};

const SESSION_KEY: [u8; 32] = [9; 32];
const USER_ID: u32 = 7;

type Session = Framed<TcpStream, SessionCodec>;

/// Accepts a connection and runs the server side of the handshake for `key`.
async fn accept(listener: &TcpListener, key: &AccountKey) -> (Session, Vec<u8>) {
    let (stream, _) = listener.accept().await.unwrap();
    let mut framed = Framed::new(stream, HandshakeCodec::new());
    let Some(Ok(Handshake::Hello { hello: client, .. })) = framed.next().await else {
        panic!("no client hello");
    };
    let hello = Hello {
        sub_protocol: SUB_PROTOCOL_KDX,
        version: VERSION,
        sub_version: SERVER_SUB_VERSION,
        challenge: 100,
        nonce: [5; 12],
        name: Vec::new(),
    };
    framed
        .send(Handshake::Hello {
            hello: hello.clone(),
            key: 0x0bad_cafe,
        })
        .await
        .unwrap();
    let Some(Ok(Handshake::Proof(proof))) = framed.next().await else {
        panic!("no client proof");
    };
    proof
        .verify(key, &hello.nonce, &fake_response(hello.challenge))
        .unwrap();
    let ours = Proof::new_ok(key, &client.nonce, fake_response(client.challenge));
    framed.send(Handshake::Proof(ours)).await.unwrap();
    (framed.map_codec(|_| SessionCodec::server(key)), client.name)
}

async fn recv(session: &mut Session) -> (Header, Bytes) {
    match session.next().await.unwrap().unwrap() {
        Frame::Message(header, payload) => (header, payload),
        other => panic!("expected a message, got {other:?}"),
    }
}

async fn send(session: &mut Session, txid: u32, kind: MessageType, reply: bool, payload: Vec<u8>) {
    let header = Header {
        txid,
        kind,
        flags: if reply { Header::FLAG_REPLY } else { 0 },
        len: 0,
    };
    session
        .send(Frame::Message(header, payload.into()))
        .await
        .unwrap();
}

/// Logs a client in to a server task that runs `serve` after the login replies.
async fn login<F>(serve: impl FnOnce(TcpListener, Session) -> F + Send + 'static) -> Client
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut session, name) = accept(&listener, &AccountKey::from_password(b"pw")).await;
        assert_eq!(name, b"admin");
        let (header, payload) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::SET_IDENTITY);
        let (me, _) = UserRecord::decode(&payload).unwrap();
        // An event before the replies must wait in the queue.
        send(
            &mut session,
            1,
            MessageType::RECEIVE_CHAT_TEXT,
            false,
            vec![1],
        )
        .await;
        let users = UserRecord::encode_list(&[UserRecord {
            user_id: USER_ID,
            ..me
        }])
        .unwrap();
        send(
            &mut session,
            header.txid,
            MessageType::USER_LIST,
            true,
            users,
        )
        .await;
        let info = ServerInfo {
            session_key: SESSION_KEY,
            user_id: USER_ID,
            name: b"Fake".to_vec(),
            ..ServerInfo::default()
        };
        let info = info.encode().unwrap();
        send(
            &mut session,
            header.txid,
            MessageType::SERVER_INFO,
            true,
            info,
        )
        .await;
        serve(listener, session).await;
    });
    let login = Login {
        name: b"admin".to_vec(),
        password: b"pw".to_vec(),
    };
    let me = UserRecord {
        nick: b"me".to_vec(),
        ..UserRecord::default()
    };
    Client::login_with(addr, &login, &me, Responses::Fake)
        .await
        .unwrap()
}

#[tokio::test]
async fn login_and_requests() {
    let mut client = login(|_, mut session| async move {
        let (header, _) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::GET_MY_ADDRESS);
        send(&mut session, 2, MessageType::PING, false, Vec::new()).await;
        let address = vec![127, 0, 0, 1];
        send(
            &mut session,
            header.txid,
            MessageType::YOUR_ADDRESS,
            true,
            address,
        )
        .await;

        let (header, _) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::DELETE_ACCOUNT);
        let error = Status::Error {
            code: 0x1000c,
            request: header.kind,
            module: 0,
            line: 0,
            extra: 0,
        };
        send(
            &mut session,
            header.txid,
            MessageType::STATUS,
            true,
            error.encode(),
        )
        .await;
        let (header, _) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::GET_MY_ADDRESS);
        send(
            &mut session,
            header.txid,
            MessageType::YOUR_ADDRESS,
            true,
            Vec::new(),
        )
        .await;
    })
    .await;

    assert_eq!(client.info.name, b"Fake");
    assert_eq!(client.users[0].nick, b"me");
    let address = client
        .request(MessageType::GET_MY_ADDRESS, &[])
        .await
        .unwrap();
    assert_eq!(address, [127, 0, 0, 1]);
    let err = client.delete_account(b"nobody").await.unwrap_err();
    assert_eq!(
        err.downcast_ref::<ServerError>(),
        Some(&ServerError {
            request: MessageType::DELETE_ACCOUNT,
            code: 0x1000c
        })
    );
    // The events arrive in order, after the replies that overtook them.
    let kinds = [
        client.next_event().await.unwrap().0.kind,
        client.next_event().await.unwrap().0.kind,
    ];
    assert_eq!(kinds, [MessageType::RECEIVE_CHAT_TEXT, MessageType::PING]);
}

#[tokio::test]
async fn download() {
    let (start, sink, file) = download_from(5000, 0).await;
    assert_eq!(start, 0x1000);
    assert!(sink[0x1000..] == file[0x1000..]);
}

#[tokio::test]
async fn download_starting_late() {
    // As seen once from kdx.erb.pw past 4 GiB.
    let (start, sink, file) = download_from(0x2000, 0xf000).await;
    assert_eq!(start, 0x11000);
    assert!(sink[0x11000..] == file[0x11000..]);
}

/// Downloads a 200000-byte file from `offset` off a server that starts `late` bytes past where it should.
async fn download_from(offset: u64, late: usize) -> (u64, Vec<u8>, Vec<u8>) {
    const LEN: u32 = 200_000;
    let file: Vec<u8> = (0..LEN).map(|i| (i % 251) as u8).collect();
    let served = file.clone();
    let mut client = login(move |listener, _session| async move {
        let (mut transfer, name) = accept(&listener, &AccountKey(SESSION_KEY)).await;
        assert_eq!(name, [0, 0, 0, 0, USER_ID as u8]);
        let (header, payload) = recv(&mut transfer).await;
        assert_eq!(header.kind, MessageType::DOWNLOAD_FILE);
        let request = DownloadFile::decode(&payload).unwrap();
        assert_eq!(request.path, FilePath::new(["a", "b"]));
        let start = (request.data_offset & !0xfff) as usize + late;
        let info = DownloadInfo {
            data_size: LEN.into(),
            ..DownloadInfo::default()
        };
        let info = info.encode().unwrap();
        send(
            &mut transfer,
            header.txid,
            MessageType::DOWNLOAD_FILE,
            true,
            info,
        )
        .await;
        let data = Header {
            txid: header.txid,
            kind: MessageType::DOWNLOAD_FILE_DATA,
            flags: Header::FLAG_REPLY,
            len: (served.len() - start) as u64,
        };
        transfer.feed(Frame::DataHeader(data)).await.unwrap();
        for chunk in served[start..].chunks(30_000) {
            let chunk = Bytes::copy_from_slice(chunk);
            transfer.feed(Frame::Data(chunk)).await.unwrap();
        }
        let rsrc = Header {
            txid: 99,
            flags: 0,
            len: 0,
            ..data
        };
        transfer.send(Frame::DataHeader(rsrc)).await.unwrap();
    })
    .await;

    let mut sink = Cursor::new(Vec::new());
    let path = FilePath::new(["a", "b"]);
    let (info, start) = client.download(&path, offset, &mut sink).await.unwrap();
    assert_eq!(info.data_size, LEN.into());
    (start, sink.into_inner(), file)
}

#[tokio::test]
async fn upload() {
    const LEN: usize = 100_000;
    let file: Vec<u8> = (0..LEN).map(|i| (i % 253) as u8).collect();
    let expected = file[0x2000..].to_vec();
    let mut client = login(move |listener, _session| async move {
        let (mut transfer, _) = accept(&listener, &AccountKey(SESSION_KEY)).await;
        let (header, payload) = recv(&mut transfer).await;
        assert_eq!(header.kind, MessageType::UPLOAD_FILE);
        let request = UploadFile::decode(&payload).unwrap();
        assert_eq!(
            (request.data_size, &request.strings.name[..]),
            (LEN as u64, &b"f"[..])
        );
        let reply = UploadReply {
            data_offset: 0x2000,
            rsrc_offset: u64::MAX,
        };
        let reply = reply.encode().to_vec();
        send(
            &mut transfer,
            header.txid,
            MessageType::UPLOAD_FILE,
            true,
            reply,
        )
        .await;

        let mut forks = Vec::new();
        while forks.len() < 2 {
            let Some(Ok(Frame::DataHeader(data))) = transfer.next().await else {
                panic!("no data header");
            };
            assert_eq!(data.kind, MessageType::UPLOAD_FILE_DATA);
            let mut fork = Vec::new();
            while fork.len() < data.len as usize {
                let Some(Ok(Frame::Data(chunk))) = transfer.next().await else {
                    panic!("data ended at {}", fork.len());
                };
                fork.extend_from_slice(&chunk);
            }
            forks.push(fork);
        }
        assert!(forks[0] == expected && forks[1].is_empty());
        send(
            &mut transfer,
            5,
            MessageType::UPLOAD_FILE_COMPLETE,
            false,
            Vec::new(),
        )
        .await;
    })
    .await;

    let folder = FilePath::new(["up"]);
    let mut source = Cursor::new(file);
    let start = client
        .upload(&folder, b"f", LEN as u64, &mut source)
        .await
        .unwrap();
    assert_eq!(start, 0x2000);
}

#[tokio::test]
async fn tracker() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut session, name) = accept(&listener, &AccountKey::from_password(b"")).await;
        assert!(name.is_empty());
        send(
            &mut session,
            1,
            MessageType::SERVER_GREETING,
            false,
            vec![0; 5],
        )
        .await;

        let (header, _) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::GET_TRACKER_GROUP_LIST);
        let groups = GroupListEntry::encode_list(&[GroupListEntry {
            servers: 1,
            name: b"General".to_vec(),
            ..GroupListEntry::default()
        }])
        .unwrap();
        let kind = MessageType::TRACKER_GROUP_LIST;
        send(&mut session, header.txid, kind, true, groups).await;

        let (header, payload) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::GET_TRACKER_SERVER_LIST);
        assert_eq!(GetServerList::decode(&payload).unwrap().group, b"Lab");
        let kind = MessageType::TRACKER_SERVER_LIST;
        send(&mut session, header.txid, kind, true, Vec::new()).await;

        // A guest's 720 gives the tracker's 8-byte error Status.
        let (header, _) = recv(&mut session).await;
        assert_eq!(header.kind, MessageType::GET_TRACKERSERV_STATISTICS);
        let status = vec![0, 1, 0, 0x14, 0, 0, 2, 0xd0];
        send(&mut session, header.txid, MessageType::STATUS, true, status).await;
    });

    let mut tracker = Tracker::login_with(addr, &Login::default(), &Responses::Fake)
        .await
        .unwrap();
    let groups = tracker.groups().await.unwrap();
    assert_eq!(
        (groups[0].servers, &groups[0].name[..]),
        (1, &b"General"[..])
    );
    assert!(tracker.servers(b"Lab").await.unwrap().is_empty());
    let err = tracker.stats().await.unwrap_err();
    assert_eq!(
        err.downcast_ref::<ServerError>(),
        Some(&ServerError {
            request: MessageType::GET_TRACKERSERV_STATISTICS,
            code: 0x10014
        })
    );
    let (greeting, _) = tracker.next_event().await.unwrap();
    assert_eq!(greeting.kind, MessageType::SERVER_GREETING);
}

//! Transfers to and from local files against the mock.

mod common;

use kdx_client::mock::Step;
use kdx_client::{kind_for, Progress, ServerError};
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, FileStrings, UploadFile, UploadReply};
use kdx_proto::msg::{FilePath, Status};
use kdx_proto::session::MessageType;

fn info(size: u64) -> DownloadInfo {
    DownloadInfo {
        data_size: size,
        strings: FileStrings {
            name: b"big.bin".to_vec(),
            second: b"text/plain".to_vec(),
            ..FileStrings::default()
        },
        ..DownloadInfo::default()
    }
}

fn download_script(size: u64, fork: Vec<u8>) -> Vec<Step> {
    vec![
        Step::Expect(MessageType::DOWNLOAD_FILE),
        Step::Reply(
            MessageType::DOWNLOAD_FILE_REPLY,
            info(size).encode().unwrap(),
        ),
        Step::DataReply {
            kind: MessageType::DOWNLOAD_FILE_DATA,
            reply: true,
            bytes: fork,
        },
        Step::DataReply {
            kind: MessageType::DOWNLOAD_FILE_DATA,
            reply: false,
            bytes: Vec::new(),
        },
    ]
}

fn body(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 251) as u8).collect()
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "kdx-client-transfer-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn download_writes_fork_bytes() {
    let data = body(5000);
    let (server, mut client) =
        common::client_with_extra(vec![], vec![download_script(5000, data.clone())]).await;
    let dest = tempdir().join("big.bin");
    let mut seen: Vec<Progress> = Vec::new();
    let got = client
        .download_to(&FilePath::new(["big.bin"]), &dest, false, |p| seen.push(p))
        .await
        .unwrap();
    assert_eq!(got.data_size, 5000);
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert_eq!(
        seen.last().unwrap(),
        &Progress {
            done: 5000,
            total: 5000
        }
    );
    drop(client);
    server.finish().await;
    let req = DownloadFile::decode(&server.payloads(MessageType::DOWNLOAD_FILE)[0]).unwrap();
    assert_eq!(req.data_offset, 0);
    assert_eq!(req.path, FilePath::new(["big.bin"]));
}

#[tokio::test]
async fn download_resume_from_rounded_offset() {
    // The local file has 0x1000 + 56 bytes; the server rounds the offset
    // down to 0x1000 and sends the rest from there.
    let data = body(9000);
    let (server, mut client) =
        common::client_with_extra(vec![], vec![download_script(9000, data[0x1000..].to_vec())])
            .await;
    let dest = tempdir().join("big.bin");
    let mut partial = data[..0x1000].to_vec();
    partial.extend_from_slice(b"garbage after the page boundary that the server resends");
    std::fs::write(&dest, &partial).unwrap();
    client
        .download_to(&FilePath::new(["big.bin"]), &dest, true, |_| {})
        .await
        .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    drop(client);
    server.finish().await;
    let req = DownloadFile::decode(&server.payloads(MessageType::DOWNLOAD_FILE)[0]).unwrap();
    assert_eq!(req.data_offset, partial.len() as u64);
}

#[tokio::test]
async fn download_resume_at_end_leaves_file_intact() {
    let (server, mut client) =
        common::client_with_extra(vec![], vec![download_script(3, Vec::new())]).await;
    let dest = tempdir().join("tiny.txt");
    std::fs::write(&dest, b"abc").unwrap();
    client
        .download_to(&FilePath::new(["tiny.txt"]), &dest, true, |_| {})
        .await
        .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), b"abc");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn download_without_resume_overwrites() {
    let (server, mut client) =
        common::client_with_extra(vec![], vec![download_script(3, b"xyz".to_vec())]).await;
    let dest = tempdir().join("tiny.txt");
    std::fs::write(&dest, b"a much longer old file").unwrap();
    client
        .download_to(&FilePath::new(["tiny.txt"]), &dest, false, |_| {})
        .await
        .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), b"xyz");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn download_into_a_directory_uses_the_server_name() {
    let (server, mut client) =
        common::client_with_extra(vec![], vec![download_script(3, b"xyz".to_vec())]).await;
    let dir = tempdir();
    let written = client
        .download_to_dir(&FilePath::new(["Uploads", "big.bin"]), &dir, false, |_| {})
        .await
        .unwrap();
    assert_eq!(written, dir.join("big.bin"));
    assert_eq!(std::fs::read(&written).unwrap(), b"xyz");
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn download_resume_refuses_a_local_file_longer_than_the_remote() {
    let (server, mut client) =
        common::client_with_extra(vec![], vec![download_script(3, Vec::new())]).await;
    let dest = tempdir().join("tiny.txt");
    std::fs::write(&dest, b"abcdefgh").unwrap();
    let err = client
        .download_to(&FilePath::new(["tiny.txt"]), &dest, true, |_| {})
        .await
        .unwrap_err();
    assert!(err.to_string().contains("longer"), "{err:#}");
    assert_eq!(std::fs::read(&dest).unwrap(), b"abcdefgh", "left alone");
    drop(client);
    server.finish().await;
}

fn upload_script(offset: u64) -> Vec<Step> {
    let reply = UploadReply {
        data_offset: offset,
        rsrc_offset: u64::MAX,
        ..UploadReply::default()
    };
    vec![
        Step::Expect(MessageType::UPLOAD_FILE),
        Step::Reply(MessageType::UPLOAD_FILE_REPLY, reply.encode().to_vec()),
        Step::ExpectData,
        Step::ExpectData,
        Step::Send(MessageType::UPLOAD_FILE_COMPLETE, Vec::new()),
    ]
}

#[tokio::test]
async fn upload_sends_417_for_each_fork_and_waits_418() {
    let data = body(70_000);
    let (server, mut client) = common::client_with_extra(vec![], vec![upload_script(0)]).await;
    let local = tempdir().join("big.bin");
    std::fs::write(&local, &data).unwrap();
    let mut last = Progress::default();
    client
        .upload_from(
            &local,
            &FilePath::new(["Uploads"]),
            b"big.bin",
            |p| last = p,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        last,
        Progress {
            done: 70_000,
            total: 70_000
        }
    );
    drop(client);
    server.finish().await;
    let req = UploadFile::decode(&server.payloads(MessageType::UPLOAD_FILE)[0]).unwrap();
    assert_eq!((req.data_size, req.rsrc_size), (70_000, 0));
    assert_eq!(req.strings.name, b"big.bin");
    assert_eq!(req.strings.second, b"exttype/bin");
    assert_eq!(req.folder, FilePath::new(["Uploads"]));
    let captured = server.captured.lock().unwrap();
    assert_eq!(captured.data.len(), 2);
    assert_eq!(captured.data[0].0, 70_000);
    assert_eq!(captured.data[0].1, data);
    assert_eq!(captured.data[1].0, 0);
}

#[tokio::test]
async fn upload_resumes_from_the_offered_offset() {
    // The lab's resume scenario: 150000 bytes cut short leave a .prt that 416
    // offers back at 0x20000, a multiple of 0x1000.
    let data = body(0x30000);
    let (server, mut client) =
        common::client_with_extra(vec![], vec![upload_script(0x20000)]).await;
    let local = tempdir().join("big.bin");
    std::fs::write(&local, &data).unwrap();
    let mut first = None;
    client
        .upload_from(
            &local,
            &FilePath::new(["Uploads"]),
            b"big.bin",
            |p| {
                first.get_or_insert(p);
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        first,
        Some(Progress {
            done: 0x20000,
            total: 0x30000
        })
    );
    drop(client);
    server.finish().await;
    let captured = server.captured.lock().unwrap();
    assert_eq!(captured.data[0].0, 0x30000 - 0x20000);
    assert_eq!(captured.data[0].1, data[0x20000..]);
}

#[tokio::test]
async fn upload_stop_after_closes_early() {
    let data = body(70_000);
    let reply = UploadReply {
        rsrc_offset: u64::MAX,
        ..UploadReply::default()
    };
    let (server, mut client) = common::client_with_extra(
        vec![],
        vec![vec![
            Step::Expect(MessageType::UPLOAD_FILE),
            Step::Reply(MessageType::UPLOAD_FILE_REPLY, reply.encode().to_vec()),
            Step::ExpectData,
        ]],
    )
    .await;
    let local = tempdir().join("big.bin");
    std::fs::write(&local, &data).unwrap();
    let mut last = Progress::default();
    let err = client
        .upload_from(
            &local,
            &FilePath::new(["Uploads"]),
            b"big.bin",
            |p| last = p,
            Some(20_000),
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("stopped"), "{err:#}");
    assert_eq!(
        last.done, 20_000,
        "stop_after is exact so the lab can predict 416's offset"
    );
    drop(client);
    // The mock's ExpectData sees the stream end early and records a failure;
    // what matters is that exactly 20_000 bytes reached it.
    server.wait().await;
    let captured = server.captured.lock().unwrap();
    assert_eq!(captured.failures.len(), 1, "{:?}", captured.failures);
    assert_eq!(captured.data[0].1, data[..20_000]);
}

#[tokio::test]
async fn upload_name_exists_is_a_server_error() {
    let status = Status::Error {
        code: 0x30073,
        request: MessageType::UPLOAD_FILE,
        module: 0,
        line: 0,
        extra: 0,
        value_14: 0,
    };
    let (server, mut client) = common::client_with_extra(
        vec![],
        vec![vec![
            Step::Expect(MessageType::UPLOAD_FILE),
            Step::Reply(MessageType::STATUS, status.encode()),
        ]],
    )
    .await;
    let local = tempdir().join("x.txt");
    std::fs::write(&local, b"x").unwrap();
    let err = client
        .upload_from(&local, &FilePath::default(), b"x.txt", |_| {}, None)
        .await
        .unwrap_err();
    assert_eq!(
        err.downcast_ref::<ServerError>().map(|e| e.code),
        Some(0x30073)
    );
    drop(client);
    server.finish().await;
}

#[tokio::test]
async fn upload_resume_refuses_a_server_partial_larger_than_the_local_file() {
    // The server holds 0x8000 bytes of an earlier upload; the local file is
    // now 3 bytes. Sending nothing would "finish" the wrong file.
    let reply = UploadReply {
        data_offset: 0x8000,
        rsrc_offset: u64::MAX,
        ..UploadReply::default()
    };
    let (server, mut client) = common::client_with_extra(
        vec![],
        vec![vec![
            Step::Expect(MessageType::UPLOAD_FILE),
            Step::Reply(MessageType::UPLOAD_FILE_REPLY, reply.encode().to_vec()),
        ]],
    )
    .await;
    let local = tempdir().join("x.bin");
    std::fs::write(&local, b"abc").unwrap();
    let err = client
        .upload_from(&local, &FilePath::new(["Uploads"]), b"x.bin", |_| {}, None)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("longer than the local file"),
        "{err:#}"
    );
    drop(client);
    server.finish().await;
    assert!(
        server.captured.lock().unwrap().data.is_empty(),
        "no data was sent"
    );
}

#[test]
fn kind_for_names_the_extension() {
    assert_eq!(kind_for(b"big.BIN"), b"exttype/bin");
    assert_eq!(kind_for(b"README"), b"");
    assert_eq!(kind_for(b".hidden"), b"");
    assert_eq!(kind_for(b"trailing."), b"");
}

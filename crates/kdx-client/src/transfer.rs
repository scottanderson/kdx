//! File transfers: 412 to 418, each on its own connection.
//!
//! [`Client::download`] and [`Client::upload`] move bytes between the server
//! and any async reader or writer; [`Client::download_to`] and
//! [`Client::upload_from`] do the same with local files, with progress and
//! resume.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

use anyhow::{bail, Context, Result};
use kdx_proto::msg::transfer::{DownloadFile, DownloadInfo, FileStrings, UploadFile, UploadReply};
use kdx_proto::msg::FilePath;
use kdx_proto::session::{Header, MessageType};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt, AsyncWrite};

use crate::client::{status_error, wait_reply};
use crate::Client;

/// How far a transfer has come, in bytes of the data fork.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    /// Bytes in place so far, counting what a resume skipped.
    pub done: u64,
    /// The fork's size.
    pub total: u64,
}

/// The type string of an upload: `exttype/<ext>` in lower case, as the
/// server itself labels files, or empty for a name without an extension.
pub fn kind_for(name: &[u8]) -> Vec<u8> {
    match name.iter().rposition(|&b| b == b'.') {
        Some(i) if i > 0 && i + 1 < name.len() => {
            let mut out = b"exttype/".to_vec();
            out.extend(name[i + 1..].iter().map(u8::to_ascii_lowercase));
            out
        }
        _ => Vec::new(),
    }
}

impl Client {
    /// Downloads a file's data fork from `offset` into `sink` on a new transfer connection.
    ///
    /// The server resumes at `offset` rounded down to 0x1000, or so it
    /// should. The data is written at the start the server's length implies,
    /// which is also returned. An `offset` past the end of the file is an
    /// error.
    pub async fn download(
        &mut self,
        path: &FilePath,
        offset: u64,
        sink: &mut (impl AsyncWrite + AsyncSeek + Unpin),
    ) -> Result<(DownloadInfo, u64)> {
        self.download_with(path, offset, sink, &mut |_| {}).await
    }

    async fn download_with(
        &mut self,
        path: &FilePath,
        offset: u64,
        sink: &mut (impl AsyncWrite + AsyncSeek + Unpin),
        on_progress: &mut dyn FnMut(Progress),
    ) -> Result<(DownloadInfo, u64)> {
        let mut conn = self.attach().await?;
        let request = DownloadFile {
            data_offset: offset,
            rsrc_offset: 0,
            options: 0,
            path: path.clone(),
            ..DownloadFile::default()
        };
        let txid = conn
            .send(MessageType::DOWNLOAD_FILE, &request.encode()?)
            .await?;
        let reply = wait_reply(
            &mut conn,
            MessageType::DOWNLOAD_FILE,
            txid,
            &mut VecDeque::new(),
        )
        .await?;
        let info = DownloadInfo::decode(&reply)?;
        if offset > info.data_size {
            bail!(
                "the local file is longer than the server's ({offset} > {} bytes); \
                 remove it or download without resume",
                info.data_size
            );
        }

        let data = conn.recv_data_header().await?;
        if data.kind != MessageType::DOWNLOAD_FILE_DATA || data.txid != txid {
            bail!("expected the data fork, got {}", data.kind);
        }
        let start = info
            .data_size
            .checked_sub(data.len)
            .context("more data than the file holds")?;
        sink.seek(std::io::SeekFrom::Start(start)).await?;
        let mut counting = Counting {
            inner: sink,
            done: start,
            total: info.data_size,
            on_progress,
        };
        counting.report();
        conn.recv_data_body(&data, &mut counting).await?;
        conn.recv_data(&mut tokio::io::sink()).await?;
        Ok((info, start))
    }

    /// Downloads a file's data fork to `dest`, reporting progress.
    ///
    /// With `resume`, the download asks from the length of the file at
    /// `dest`; the server rounds down to 0x1000 and the tail is rewritten.
    /// Without it, the file is truncated first.
    pub async fn download_to(
        &mut self,
        path: &FilePath,
        dest: &Path,
        resume: bool,
        mut on_progress: impl FnMut(Progress),
    ) -> Result<DownloadInfo> {
        let existing = if resume {
            tokio::fs::metadata(dest).await.map_or(0, |m| m.len())
        } else {
            0
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(!resume)
            .open(dest)
            .await
            .with_context(|| format!("opening {}", dest.display()))?;
        let (info, _) = self
            .download_with(path, existing, &mut file, &mut on_progress)
            .await?;
        file.set_len(info.data_size).await?;
        Ok(info)
    }

    /// [`Self::download_to`] into `dir`, named after the last path component.
    pub async fn download_to_dir(
        &mut self,
        path: &FilePath,
        dir: &Path,
        resume: bool,
        on_progress: impl FnMut(Progress),
    ) -> Result<PathBuf> {
        let name = path.0.last().context("cannot download the base folder")?;
        let dest = dir.join(String::from_utf8_lossy(name).as_ref());
        self.download_to(path, &dest, resume, on_progress).await?;
        Ok(dest)
    }

    /// Uploads `len` bytes from `source` as `name` into `folder` on a new transfer connection.
    ///
    /// If an earlier upload of the same name was cut short, the server resumes
    /// it; `source` is then read from the returned offset onward, so it must
    /// support skipping there. A resume offset past `len` is an error: the
    /// server's partial is not this file.
    pub async fn upload(
        &mut self,
        folder: &FilePath,
        name: &[u8],
        len: u64,
        source: &mut (impl AsyncRead + AsyncSeek + Unpin),
    ) -> Result<u64> {
        self.upload_with(folder, name, len, source, &mut |_| {}, None)
            .await
    }

    async fn upload_with(
        &mut self,
        folder: &FilePath,
        name: &[u8],
        len: u64,
        source: &mut (impl AsyncRead + AsyncSeek + Unpin),
        on_progress: &mut dyn FnMut(Progress),
        stop_after: Option<u64>,
    ) -> Result<u64> {
        let mut conn = self.attach().await?;
        let request = UploadFile {
            data_size: len,
            rsrc_size: 0,
            options: 0,
            strings: FileStrings {
                name: name.to_vec(),
                second: kind_for(name),
                ..FileStrings::default()
            },
            folder: folder.clone(),
            ..UploadFile::default()
        };
        let txid = conn
            .send(MessageType::UPLOAD_FILE, &request.encode()?)
            .await?;
        let reply = wait_reply(
            &mut conn,
            MessageType::UPLOAD_FILE,
            txid,
            &mut VecDeque::new(),
        )
        .await?;
        let start = UploadReply::decode(&reply)?.data_offset;
        if start > len {
            bail!(
                "the server's partial upload ({start} bytes) is longer than the local file \
                 ({len} bytes); it is not this file"
            );
        }

        source.seek(std::io::SeekFrom::Start(start)).await?;
        let left = len - start;
        if left > Header::MAX_LEN {
            bail!("file too large for one transfer");
        }
        on_progress(Progress {
            done: start,
            total: len,
        });
        conn.send_data_with(
            MessageType::UPLOAD_FILE_DATA,
            left,
            source,
            |sent| {
                on_progress(Progress {
                    done: start + sent,
                    total: len,
                });
            },
            stop_after,
        )
        .await?;
        conn.send_data(MessageType::UPLOAD_FILE_DATA, 0, &mut tokio::io::empty())
            .await?;
        loop {
            let (header, payload) = conn.recv().await?;
            match header.kind {
                MessageType::UPLOAD_FILE_COMPLETE => return Ok(start),
                MessageType::STATUS => bail!(status_error(MessageType::UPLOAD_FILE, &payload)),
                _ => {}
            }
        }
    }

    /// Uploads the file at `local` as `name` into `folder`, reporting
    /// progress; a partial upload of the same name on the server is resumed.
    ///
    /// `stop_after` cuts the stream after that many bytes and fails, to leave
    /// a partial upload on the server for a test of resume.
    pub async fn upload_from(
        &mut self,
        local: &Path,
        folder: &FilePath,
        name: &[u8],
        mut on_progress: impl FnMut(Progress),
        stop_after: Option<u64>,
    ) -> Result<()> {
        let mut file = File::open(local)
            .await
            .with_context(|| format!("opening {}", local.display()))?;
        let len = file.metadata().await?.len();
        self.upload_with(folder, name, len, &mut file, &mut on_progress, stop_after)
            .await?;
        Ok(())
    }
}

/// A writer that reports the bytes written through it.
struct Counting<'a, W> {
    inner: &'a mut W,
    done: u64,
    total: u64,
    on_progress: &'a mut dyn FnMut(Progress),
}

impl<W> Counting<'_, W> {
    fn report(&mut self) {
        (self.on_progress)(Progress {
            done: self.done,
            total: self.total,
        });
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for Counting<'_, W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let written = Pin::new(&mut *this.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = &written {
            this.done += *n as u64;
            this.report();
        }
        written
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.get_mut().inner).poll_shutdown(cx)
    }
}

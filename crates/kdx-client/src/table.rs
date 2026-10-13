//! Finding a challenge table: a given file, the cache, or a download.
//!
//! The tables are Haxial's text, so they ship with no build of this project.
//! [`load`] reads one from a KDX program file, a saved table, or a zip archive
//! of a KDX program, and [`fetch`] downloads the smallest known archive that
//! holds it from the Internet Archive. [`save_to_cache`] keeps it for later
//! runs. Each function works on either [`Table`]: the [`ChallengeTable`] of
//! every KDX connection, or the [`GConsoleTable`] that only the clients hold.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use kdx_proto::challenge::ChallengeError;
use kdx_proto::{ChallengeTable, GConsoleTable};
use sha2::{Digest, Sha256};

/// A challenge table this module can find, cache, and download.
pub trait Table: Sized {
    /// What messages call the table.
    const NAME: &'static str;
    /// The table's file name in the cache directory: its service with the `%` dropped.
    const CACHE_FILE: &'static str;
    /// The file name earlier versions cached the table under, in the directory above.
    const LEGACY_CACHE_FILE: &'static str;
    /// The smallest archive known to hold the table.
    const FETCH_URL: &'static str;
    /// SHA-256 of the archive at [`FETCH_URL`](Self::FETCH_URL), from `vendor/MANIFEST.md`.
    const FETCH_SHA256: &'static str;

    /// Loads a saved table and verifies its checksum.
    fn from_bytes(bytes: &[u8]) -> Result<Self, ChallengeError>;
    /// Finds and verifies the table inside an uncompressed KDX program file.
    fn extract(file: &[u8]) -> Result<Self, ChallengeError>;
    /// The table bytes.
    fn as_bytes(&self) -> &[u8];
}

impl Table for ChallengeTable {
    const NAME: &'static str = "KDX challenge table";
    const CACHE_FILE: &'static str = "kdx.bin";
    const LEGACY_CACHE_FILE: &'static str = "challenge-table.bin";
    /// Tracker 1.110 for Windows, 248 KB.
    const FETCH_URL: &'static str = "https://web.archive.org/web/20051014065628id_/http://www.haxial.com:80/download/KDXTrackerServ1110-Win.zip";
    const FETCH_SHA256: &'static str =
        "8f50ef0cd4f82fc5b071340932c498a9df3556ed6cff6f23ecd6c36c43683583";

    fn from_bytes(bytes: &[u8]) -> Result<Self, ChallengeError> {
        Self::from_bytes(bytes)
    }
    fn extract(file: &[u8]) -> Result<Self, ChallengeError> {
        Self::extract(file)
    }
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Table for GConsoleTable {
    const NAME: &'static str = "GConsole challenge table";
    const CACHE_FILE: &'static str = "hgc.bin";
    const LEGACY_CACHE_FILE: &'static str = "gconsole-table.bin";
    /// Client 1.600 for Windows, 793 KB; no server or tracker holds this table.
    const FETCH_URL: &'static str = "https://web.archive.org/web/20051014063319id_/http://www.haxial.com:80/download/KDXClient1600-Win.zip";
    const FETCH_SHA256: &'static str =
        "cc6ec09bad1969e354ed5778ad594a0f8639ae8e3aa05e169916cc4b8d4f038b";

    fn from_bytes(bytes: &[u8]) -> Result<Self, ChallengeError> {
        Self::from_bytes(bytes)
    }
    fn extract(file: &[u8]) -> Result<Self, ChallengeError> {
        Self::extract(file)
    }
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Largest archive or inflated file this module reads.
const MAX_FILE_LEN: u64 = 32 << 20;

/// Where the table is cached: in `$XDG_CACHE_HOME/kdx/table` or `~/.cache/kdx/table`.
pub fn cache_path<T: Table>() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(dir.join("kdx").join("table").join(T::CACHE_FILE))
}

/// The cached table, if there is a valid one.
pub fn cached<T: Table>() -> Option<T> {
    let path = cache_path::<T>()?;
    migrate::<T>(&path);
    let bytes = std::fs::read(path).ok()?;
    T::from_bytes(&bytes).ok()
}

/// Moves a table from its legacy cache path to `path`, or removes it if `path` holds a valid one.
fn migrate<T: Table>(path: &Path) {
    let Some(legacy) = path.parent().and_then(Path::parent) else {
        return;
    };
    let legacy = legacy.join(T::LEGACY_CACHE_FILE);
    let Ok(old) = std::fs::read(&legacy) else {
        return;
    };
    let valid = |bytes: &[u8]| T::from_bytes(bytes).is_ok();
    if std::fs::read(path).is_ok_and(|new| valid(&new)) {
        let _ = std::fs::remove_file(&legacy);
    } else if valid(&old) && std::fs::create_dir_all(path.parent().expect("has a parent")).is_ok() {
        let _ = std::fs::rename(&legacy, path);
    }
}

/// Saves the table to the cache and returns its path.
pub fn save_to_cache<T: Table>(table: &T) -> Result<PathBuf> {
    let path = cache_path::<T>().context("no cache directory: set XDG_CACHE_HOME or HOME")?;
    std::fs::create_dir_all(path.parent().expect("has a parent"))?;
    std::fs::write(&path, table.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    migrate::<T>(&path);
    Ok(path)
}

/// Reads the table from a saved table, a KDX program file, or a zip archive of one.
pub fn load<T: Table>(path: &Path) -> Result<T> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_LEN).read_to_end(&mut bytes)?;
    from_file_bytes(&bytes).with_context(|| format!("{}", path.display()))
}

/// Downloads the archive at [`Table::FETCH_URL`], checks it, and extracts the table.
pub fn fetch<T: Table>() -> Result<T> {
    let mut response = ureq::get(T::FETCH_URL).call().context("downloading")?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_FILE_LEN)
        .read_to_vec()
        .context("downloading")?;
    let digest = Sha256::digest(&bytes);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    if hex != T::FETCH_SHA256 {
        bail!(
            "the downloaded archive's SHA-256 is {hex}, expected {}",
            T::FETCH_SHA256
        );
    }
    from_file_bytes(&bytes)
}

fn from_file_bytes<T: Table>(bytes: &[u8]) -> Result<T> {
    if let Ok(table) = T::from_bytes(bytes) {
        return Ok(table);
    }
    if let Ok(table) = T::extract(bytes) {
        return Ok(table);
    }
    if bytes.starts_with(b"PK\x03\x04") {
        for entry in zip_entries(bytes)? {
            if let Ok(table) = T::extract(&entry?) {
                return Ok(table);
            }
        }
        bail!("no file in the archive holds the {}", T::NAME);
    }
    bail!("not a KDX program, a saved {}, or a zip archive", T::NAME);
}

/// Inflates each stored or deflated file in a zip archive, using its central directory.
fn zip_entries(zip: &[u8]) -> Result<impl Iterator<Item = Result<Vec<u8>>> + '_> {
    let u16_at = |at: usize| -> Result<usize> {
        let b = zip.get(at..at + 2).context("truncated zip")?;
        Ok(usize::from(u16::from_le_bytes([b[0], b[1]])))
    };
    let u32_at = |at: usize| -> Result<usize> {
        let b = zip.get(at..at + 4).context("truncated zip")?;
        Ok(u32::from_le_bytes(b.try_into().expect("4 bytes")) as usize)
    };
    let eocd = zip
        .windows(4)
        .rposition(|w| w == b"PK\x05\x06")
        .context("no zip central directory")?;
    let count = u16_at(eocd + 10)?;
    let mut at = u32_at(eocd + 16)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        if zip.get(at..at + 4) != Some(b"PK\x01\x02") {
            bail!("bad zip central directory entry");
        }
        let method = u16_at(at + 10)?;
        let packed = u32_at(at + 20)?;
        let local = u32_at(at + 42)?;
        at += 46 + u16_at(at + 28)? + u16_at(at + 30)? + u16_at(at + 32)?;
        if zip.get(local..local + 4) != Some(b"PK\x03\x04") {
            bail!("bad zip local header");
        }
        let start = local + 30 + u16_at(local + 26)? + u16_at(local + 28)?;
        let data = zip.get(start..start + packed).context("truncated zip")?;
        entries.push((method, data));
    }
    Ok(entries
        .into_iter()
        .filter_map(|(method, data)| match method {
            0 => Some(Ok(data.to_vec())),
            8 => {
                let mut out = Vec::new();
                let inflated = flate2::read::DeflateDecoder::new(data)
                    .take(MAX_FILE_LEN)
                    .read_to_end(&mut out);
                Some(inflated.map(|_| out).context("inflating a zip entry"))
            }
            _ => None,
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VENDOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/haxial");

    #[test]
    fn loads_programs_and_archives() {
        // The table is not in the repository; skip unless the lab binaries are present.
        let zip = Path::new(VENDOR).join("KDXTrackerServ1110-Win/KDXTrackerServ1110-Win.zip");
        let Ok(bytes) = std::fs::read(&zip) else {
            eprintln!("skipping: {} not found", zip.display());
            return;
        };
        let hex: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, ChallengeTable::FETCH_SHA256);
        let table: ChallengeTable = load(&zip).unwrap();
        let program =
            Path::new(VENDOR).join("KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe");
        assert_eq!(load::<ChallengeTable>(&program).unwrap(), table);
        assert!(load::<GConsoleTable>(&program).is_err());
    }

    #[test]
    fn loads_the_gconsole_table_from_clients() {
        // The table is not in the repository; skip unless the lab binaries are present.
        let zip = Path::new(VENDOR).join("KDXClient1600-Win/KDXClient1600-Win.zip");
        let Ok(bytes) = std::fs::read(&zip) else {
            eprintln!("skipping: {} not found", zip.display());
            return;
        };
        let hex: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, GConsoleTable::FETCH_SHA256);
        let table: GConsoleTable = load(&zip).unwrap();
        let program = Path::new(VENDOR)
            .join("KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe");
        assert_eq!(load::<GConsoleTable>(&program).unwrap(), table);
    }

    #[test]
    fn rejects_other_files() {
        assert!(from_file_bytes::<ChallengeTable>(b"not kdx").is_err());
        assert!(from_file_bytes::<GConsoleTable>(b"PK\x03\x04 but truncated").is_err());
    }
}

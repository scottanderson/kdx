//! Finding the challenge table: a given file, the cache, or a download.
//!
//! The table is Haxial's text, so it ships with no build of this project.
//! [`load`] reads it from a KDX program file, a saved table, or a zip archive
//! of a KDX program, and [`fetch`] downloads the smallest known archive from the
//! Internet Archive. [`save_to_cache`] keeps it for later runs.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use kdx_proto::ChallengeTable;
use sha2::{Digest, Sha256};

/// The smallest archive known to hold the table: Tracker 1.110 for Windows, 248 KB.
pub const FETCH_URL: &str = "https://web.archive.org/web/20051014065628id_/http://www.haxial.com:80/download/KDXTrackerServ1110-Win.zip";

/// SHA-256 of the archive at [`FETCH_URL`], from `vendor/MANIFEST.md`.
const FETCH_SHA256: &str = "8f50ef0cd4f82fc5b071340932c498a9df3556ed6cff6f23ecd6c36c43683583";

/// Largest archive or inflated file this module reads.
const MAX_FILE_LEN: u64 = 32 << 20;

/// Where the table is cached: `$XDG_CACHE_HOME/kdx` or `~/.cache/kdx`.
pub fn cache_path() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(dir.join("kdx").join("challenge-table.bin"))
}

/// The cached table, if there is a valid one.
pub fn cached() -> Option<ChallengeTable> {
    let bytes = std::fs::read(cache_path()?).ok()?;
    ChallengeTable::from_bytes(&bytes).ok()
}

/// Saves the table to the cache and returns its path.
pub fn save_to_cache(table: &ChallengeTable) -> Result<PathBuf> {
    let path = cache_path().context("no cache directory: set XDG_CACHE_HOME or HOME")?;
    std::fs::create_dir_all(path.parent().expect("has a parent"))?;
    std::fs::write(&path, table.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Reads the table from a saved table, a KDX program file, or a zip archive of one.
pub fn load(path: &Path) -> Result<ChallengeTable> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_LEN).read_to_end(&mut bytes)?;
    from_file_bytes(&bytes).with_context(|| format!("{}", path.display()))
}

/// Downloads the archive at [`FETCH_URL`], checks it, and extracts the table.
pub fn fetch() -> Result<ChallengeTable> {
    let mut response = ureq::get(FETCH_URL).call().context("downloading")?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_FILE_LEN)
        .read_to_vec()
        .context("downloading")?;
    let digest = Sha256::digest(&bytes);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    if hex != FETCH_SHA256 {
        bail!("the downloaded archive's SHA-256 is {hex}, expected {FETCH_SHA256}");
    }
    from_file_bytes(&bytes)
}

fn from_file_bytes(bytes: &[u8]) -> Result<ChallengeTable> {
    if let Ok(table) = ChallengeTable::from_bytes(bytes) {
        return Ok(table);
    }
    if let Ok(table) = ChallengeTable::extract(bytes) {
        return Ok(table);
    }
    if bytes.starts_with(b"PK\x03\x04") {
        for entry in zip_entries(bytes)? {
            if let Ok(table) = ChallengeTable::extract(&entry?) {
                return Ok(table);
            }
        }
        bail!("no file in the archive holds the KDX challenge table");
    }
    bail!("not a KDX program, a saved challenge table, or a zip archive");
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
        assert_eq!(hex, FETCH_SHA256);
        let table = load(&zip).unwrap();
        let program =
            Path::new(VENDOR).join("KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe");
        assert_eq!(load(&program).unwrap(), table);
    }

    #[test]
    fn rejects_other_files() {
        assert!(from_file_bytes(b"not kdx").is_err());
        assert!(from_file_bytes(b"PK\x03\x04 but truncated").is_err());
    }
}

//! Finding a challenge table: a given file, the cache, or a download.
//!
//! The stories are Haxial's text, so they ship with no build of this project.
//! [`load`] reads one from a KDX program file, a saved story, or a zip archive
//! of a KDX program, and [`fetch`] downloads the smallest known archive that
//! holds it from the Internet Archive. [`save_to_cache`] keeps it for later
//! runs. Each function works on either [`Story`]: the KDX story of every
//! connection, or the GConsole story that only the clients hold.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use kdx_proto::{ChallengeTable, Story};
use sha2::{Digest, Sha256};

/// The smallest archive known to hold the story.
pub const fn fetch_url(story: Story) -> &'static str {
    match story {
        // Tracker 1.110 for Windows, 248 KB.
        Story::Kdx => "https://web.archive.org/web/20051014065628id_/http://www.haxial.com:80/download/KDXTrackerServ1110-Win.zip",
        // Client 1.600 for Windows, 793 KB; no server or tracker holds this story.
        Story::Hgc => "https://web.archive.org/web/20051014063319id_/http://www.haxial.com:80/download/KDXClient1600-Win.zip",
    }
}

/// SHA-256 of the archive at [`fetch_url`], from `vendor/MANIFEST.md`.
pub const fn fetch_sha256(story: Story) -> &'static str {
    match story {
        Story::Kdx => "8f50ef0cd4f82fc5b071340932c498a9df3556ed6cff6f23ecd6c36c43683583",
        Story::Hgc => "cc6ec09bad1969e354ed5778ad594a0f8639ae8e3aa05e169916cc4b8d4f038b",
    }
}

/// The file name earlier versions cached the story under, in the directory above.
const fn legacy_cache_file(story: Story) -> &'static str {
    match story {
        Story::Kdx => "challenge-table.bin",
        Story::Hgc => "gconsole-table.bin",
    }
}

/// Largest archive or inflated file this module reads.
const MAX_FILE_LEN: u64 = 32 << 20;

/// Where the story is cached: `$XDG_CACHE_HOME/kdx/table` or `~/.cache/kdx/table`,
/// named for its sub-protocol with the `%` dropped, such as `kdx.bin`.
pub fn cache_path(story: Story) -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    let sub_protocol = story.sub_protocol().to_be_bytes();
    let name = String::from_utf8_lossy(&sub_protocol[1..]).to_ascii_lowercase();
    Some(dir.join("kdx").join("table").join(format!("{name}.bin")))
}

/// The cached story, if there is a valid one.
pub fn cached(story: Story) -> Option<ChallengeTable> {
    let path = cache_path(story)?;
    migrate(story, &path);
    let bytes = std::fs::read(path).ok()?;
    ChallengeTable::from_bytes(story, &bytes).ok()
}

/// Moves a story from its legacy cache path to `path`, or removes it if `path` holds a valid one.
fn migrate(story: Story, path: &Path) {
    let Some(legacy) = path.parent().and_then(Path::parent) else {
        return;
    };
    let legacy = legacy.join(legacy_cache_file(story));
    let Ok(old) = std::fs::read(&legacy) else {
        return;
    };
    let valid = |bytes: &[u8]| ChallengeTable::from_bytes(story, bytes).is_ok();
    if std::fs::read(path).is_ok_and(|new| valid(&new)) {
        let _ = std::fs::remove_file(&legacy);
    } else if valid(&old) && std::fs::create_dir_all(path.parent().expect("has a parent")).is_ok() {
        let _ = std::fs::rename(&legacy, path);
    }
}

/// Saves a story to the cache and returns its path.
pub fn save_to_cache(table: &ChallengeTable) -> Result<PathBuf> {
    let story = table.story().context("the default table is not cached")?;
    let path = cache_path(story).context("no cache directory: set XDG_CACHE_HOME or HOME")?;
    std::fs::create_dir_all(path.parent().expect("has a parent"))?;
    std::fs::write(&path, table.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    migrate(story, &path);
    Ok(path)
}

/// Reads the story from a saved story, a KDX program file, or a zip archive of one.
pub fn load(story: Story, path: &Path) -> Result<ChallengeTable> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_LEN).read_to_end(&mut bytes)?;
    from_file_bytes(story, &bytes).with_context(|| format!("{}", path.display()))
}

/// Downloads the archive at [`fetch_url`], checks it, and extracts the story.
pub fn fetch(story: Story) -> Result<ChallengeTable> {
    let mut response = ureq::get(fetch_url(story)).call().context("downloading")?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_FILE_LEN)
        .read_to_vec()
        .context("downloading")?;
    let digest = Sha256::digest(&bytes);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    if hex != fetch_sha256(story) {
        bail!(
            "the downloaded archive's SHA-256 is {hex}, expected {}",
            fetch_sha256(story)
        );
    }
    from_file_bytes(story, &bytes)
}

fn from_file_bytes(story: Story, bytes: &[u8]) -> Result<ChallengeTable> {
    if let Ok(table) = ChallengeTable::from_bytes(story, bytes) {
        return Ok(table);
    }
    if let Ok(table) = ChallengeTable::extract(story, bytes) {
        return Ok(table);
    }
    if bytes.starts_with(b"PK\x03\x04") {
        for entry in zip_entries(bytes)? {
            if let Ok(table) = ChallengeTable::extract(story, &entry?) {
                return Ok(table);
            }
        }
        bail!("no file in the archive holds the {story}");
    }
    bail!("not a KDX program, a saved {story}, or a zip archive");
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
        // The story is not in the repository; skip unless the lab binaries are present.
        let zip = Path::new(VENDOR).join("KDXTrackerServ1110-Win/KDXTrackerServ1110-Win.zip");
        let Ok(bytes) = std::fs::read(&zip) else {
            eprintln!("skipping: {} not found", zip.display());
            return;
        };
        let hex: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, fetch_sha256(Story::Kdx));
        let table = load(Story::Kdx, &zip).unwrap();
        let program =
            Path::new(VENDOR).join("KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe");
        assert_eq!(load(Story::Kdx, &program).unwrap(), table);
        assert!(load(Story::Hgc, &program).is_err());
    }

    #[test]
    fn loads_the_gconsole_story_from_clients() {
        // The story is not in the repository; skip unless the lab binaries are present.
        let zip = Path::new(VENDOR).join("KDXClient1600-Win/KDXClient1600-Win.zip");
        let Ok(bytes) = std::fs::read(&zip) else {
            eprintln!("skipping: {} not found", zip.display());
            return;
        };
        let hex: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, fetch_sha256(Story::Hgc));
        let table = load(Story::Hgc, &zip).unwrap();
        let program = Path::new(VENDOR)
            .join("KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe");
        assert_eq!(load(Story::Hgc, &program).unwrap(), table);
    }

    #[test]
    fn names_cache_files_for_the_sub_protocol() {
        let names =
            Story::ALL.map(|story| cache_path(story).map(|p| p.file_name().unwrap().to_owned()));
        assert_eq!(names, [Some("kdx.bin".into()), Some("hgc.bin".into())]);
    }

    #[test]
    fn rejects_other_files() {
        assert!(from_file_bytes(Story::Kdx, b"not kdx").is_err());
        assert!(from_file_bytes(Story::Hgc, b"PK\x03\x04 but truncated").is_err());
    }
}

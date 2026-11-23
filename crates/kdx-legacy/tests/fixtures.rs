//! Round trips over every committed fixture: `encode(decode(file)) == file`.
//!
//! The walk decodes each file under `tests/fixtures/` by its name: `*.dat`
//! under `accounts/`, `news/` and `catalog/` and `Accounts.dat`; `*.stg` by
//! magic; `History/*.txt`; `News Archives/**.txt`. READMEs are skipped.

use std::fs;
use std::path::{Path, PathBuf};

use kdx_legacy::accounts::{decode_accounts, encode_accounts};
use kdx_legacy::catalog::{decode_catalog, encode_catalog};
use kdx_legacy::client_stg::{decode_client_stg, encode_client_stg};
use kdx_legacy::history::parse_history;
use kdx_legacy::news::{decode_news, encode_news};
use kdx_legacy::news_archive::parse_news_archive;
use kdx_legacy::server_stg::{decode_server_stg, encode_server_stg};
use kdx_legacy::tracker_stg::{decode_tracker_stg, encode_tracker_stg};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn has_component(p: &Path, name: &str) -> bool {
    p.components().any(|c| c.as_os_str() == name)
}

/// Re-encodes `bytes` through the decoder its path names; `None` for a file
/// that is not a format (a README).
fn round_trip(path: &Path, bytes: &[u8]) -> Option<Vec<u8>> {
    let name = path.file_name().unwrap().to_str().unwrap();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let out = match ext {
        "md" => return None,
        "dat" if name == "Accounts.dat" || has_component(path, "accounts") => {
            encode_accounts(&decode_accounts(bytes).unwrap()).unwrap()
        }
        "dat" if has_component(path, "news") => encode_news(&decode_news(bytes).unwrap()).unwrap(),
        "dat" if has_component(path, "catalog") => encode_catalog(&decode_catalog(bytes).unwrap()),
        "stg" if bytes.starts_with(b"%KDXSSTG") => {
            encode_server_stg(&decode_server_stg(bytes).unwrap()).unwrap()
        }
        "stg" if has_component(path, "tracker") => {
            encode_tracker_stg(&decode_tracker_stg(bytes).unwrap())
        }
        "stg" => encode_client_stg(&decode_client_stg(bytes).unwrap()),
        "txt" if has_component(path, "History") => parse_history(bytes).unwrap().to_bytes(),
        "txt" if has_component(path, "News Archives") => {
            let mut again = b"\xef\xbb\xbf".to_vec();
            for e in parse_news_archive(bytes).unwrap() {
                again.extend(e.to_bytes());
            }
            again
        }
        _ => panic!("{path:?}: no decoder for this fixture"),
    };
    Some(out)
}

#[test]
fn every_fixture_round_trips_byte_exact() {
    let mut files = Vec::new();
    walk(&fixtures(), &mut files);
    files.sort();
    let mut checked = 0;
    for path in &files {
        let bytes = fs::read(path).unwrap();
        if let Some(again) = round_trip(path, &bytes) {
            assert_eq!(again, bytes, "{path:?}");
            checked += 1;
        }
    }
    assert_eq!(files.len(), 50, "{files:?}");
    assert_eq!(checked, 48, "{files:?}");
}

//! The challenge tables for the workspace's tests.
//!
//! The tables are Haxial's text, so they are not in the repository. Each
//! function reads its table from an environment variable (base64, which CI
//! sets from a secret of the same name), the lab binaries in `vendor/haxial/`,
//! or the `kdx table fetch` cache, and panics without all three.

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use kdx_client::table::{self, Table};
use kdx_proto::{ChallengeTable, GConsoleTable};

const VENDOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/haxial");

/// The KDX table, from `KDX_CHALLENGE_TABLE`, Server 1.620 for Linux, or the cache.
pub fn challenge_table() -> ChallengeTable {
    find(
        "KDX_CHALLENGE_TABLE",
        "KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe",
    )
}

/// The GConsole table, from `HGC_CHALLENGE_TABLE`, Client 1.620 beta 5 for Linux, or the cache.
pub fn gconsole_table() -> GConsoleTable {
    find(
        "HGC_CHALLENGE_TABLE",
        "KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe",
    )
}

fn find<T: Table>(var: &str, vendor: &str) -> T {
    let env = std::env::var(var).ok().filter(|v| !v.trim().is_empty());
    if let Some(b64) = env {
        let bytes = STANDARD
            .decode(b64.trim())
            .unwrap_or_else(|e| panic!("{var} is not base64: {e}"));
        return T::from_bytes(&bytes).unwrap_or_else(|e| panic!("{var}: {e}"));
    }
    if let Ok(file) = std::fs::read(format!("{VENDOR}/{vendor}")) {
        return T::extract(&file).unwrap_or_else(|e| panic!("vendor/haxial/{vendor}: {e}"));
    }
    table::cached().unwrap_or_else(|| {
        panic!(
            "no {} in {var}, vendor/haxial, or the kdx cache; run `cargo run --bin kdx -- table fetch`",
            T::NAME
        )
    })
}

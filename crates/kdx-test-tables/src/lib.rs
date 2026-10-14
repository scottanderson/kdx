//! The challenge tables for the workspace's tests.
//!
//! The stories are Haxial's text, so they are not in the repository. [`table`]
//! reads one from an environment variable (base64, which CI sets from a secret
//! of the same name), the lab binaries in `vendor/haxial/`, or the `kdx table
//! fetch` cache, and panics without all three.

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use kdx_client::table;
use kdx_proto::{ChallengeTable, Story};

const VENDOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/haxial");

/// The story, from `KDX_CHALLENGE_TABLE` or `HGC_CHALLENGE_TABLE`, a lab binary, or the cache.
pub fn table(story: Story) -> ChallengeTable {
    let var = env_var(story);
    if let Some(b64) = std::env::var(&var).ok().filter(|v| !v.trim().is_empty()) {
        let bytes = STANDARD
            .decode(b64.trim())
            .unwrap_or_else(|e| panic!("{var} is not base64: {e}"));
        return ChallengeTable::from_bytes(story, &bytes).unwrap_or_else(|e| panic!("{var}: {e}"));
    }
    let program = vendor_program(story);
    if let Ok(file) = std::fs::read(format!("{VENDOR}/{program}")) {
        return ChallengeTable::extract(story, &file)
            .unwrap_or_else(|e| panic!("vendor/haxial/{program}: {e}"));
    }
    table::cached(story).unwrap_or_else(|| {
        panic!("no {story} in {var}, vendor/haxial, or the kdx cache; run `cargo run --bin kdx -- table fetch`")
    })
}

/// The story's variable: its sub-protocol with the `%` dropped, then `_CHALLENGE_TABLE`.
fn env_var(story: Story) -> String {
    let sub_protocol = story.sub_protocol();
    format!(
        "{}_CHALLENGE_TABLE",
        String::from_utf8_lossy(&sub_protocol[1..])
    )
}

/// A lab binary that holds the story: Server 1.620 or Client 1.620 beta 5 for Linux.
const fn vendor_program(story: Story) -> &'static str {
    match story {
        Story::Kdx => "KDXServer1620-Lnx/extracted/KDXServer1620-Lnx/KDXServer.lexe",
        Story::Hgc => "KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe",
    }
}

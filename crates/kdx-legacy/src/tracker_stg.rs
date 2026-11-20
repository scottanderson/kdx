//! `KDXTrackerServ.stg`: the tracker's settings, the `%KDXTSTG` blob of 711
//! and 712, scrambled with [`Stream::TRACKER_SETTINGS`]. Sections stay
//! opaque until Phase 5. See `docs/formats/tracker-stg.md`.

use kdx_proto::msg::tracker::TrackerSettings;

use crate::error::Result;
use crate::scramble::{scramble, unscramble, Stream};
use crate::server_stg::check_header;
use crate::store::{LegacyDir, ServerFile};

/// The plain file's first eight bytes.
pub const MAGIC: [u8; 8] = *b"%KDXTSTG";

/// Decodes a scrambled `KDXTrackerServ.stg`.
pub fn decode_tracker_stg(bytes: &[u8]) -> Result<TrackerSettings> {
    let plain = unscramble(bytes, Stream::TRACKER_SETTINGS);
    check_header(&plain, MAGIC, 1, ServerFile::Tracker.name())?;
    Ok(TrackerSettings::decode(&plain)?)
}

/// Encodes and scrambles a `KDXTrackerServ.stg`.
pub fn encode_tracker_stg(settings: &TrackerSettings) -> Vec<u8> {
    scramble(&settings.encode(), Stream::TRACKER_SETTINGS)
}

impl LegacyDir {
    /// Reads `KDXTrackerServ.stg` if the directory has one.
    pub fn load_tracker_settings(&self) -> Result<Option<TrackerSettings>> {
        self.read(ServerFile::Tracker)?
            .map(|b| decode_tracker_stg(&b))
            .transpose()
    }

    /// Writes `KDXTrackerServ.stg` atomically.
    pub fn save_tracker_settings(&self, settings: &TrackerSettings) -> Result<()> {
        self.save(ServerFile::Tracker, &encode_tracker_stg(settings))
    }
}

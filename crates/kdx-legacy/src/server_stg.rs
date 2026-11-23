//! `KDXServer.stg`: the server's settings, the same `%KDXSSTG` blob that 171
//! and 172 carry on the wire. See `docs/formats/server-stg.md`.

use crate::wire::DecodeError;
use kdx_proto::msg::server_info::ServerSettings;

use crate::error::{LegacyError, Result};
use crate::store::{LegacyDir, ServerFile};

/// The file's first eight bytes.
pub const MAGIC: [u8; 8] = *b"%KDXSSTG";

/// Decodes a `KDXServer.stg`.
pub fn decode_server_stg(bytes: &[u8]) -> Result<ServerSettings> {
    check_header(bytes, MAGIC, 1, ServerFile::Settings.name())?;
    Ok(ServerSettings::decode(bytes)?)
}

/// Encodes the settings; fails on a string longer than its length byte.
pub fn encode_server_stg(settings: &ServerSettings) -> Result<Vec<u8>> {
    Ok(settings.encode()?)
}

/// Refuses settings whose strings do not fit their length bytes.
pub fn validate_settings(settings: &ServerSettings) -> Result<()> {
    settings.encode().map(|_| ()).map_err(|_| {
        LegacyError::Decode(DecodeError::Invalid(
            "server settings string longer than 255 bytes",
        ))
    })
}

/// Magic and version checks shared by the `%KDX`-tagged files, with errors
/// that name the problem before the generic decoder runs.
pub(crate) fn check_header(
    bytes: &[u8],
    magic: [u8; 8],
    version: u16,
    file: &'static str,
) -> Result<()> {
    if bytes.len() < 10 {
        return Err(LegacyError::Decode(DecodeError::Truncated {
            offset: bytes.len(),
            needed: 10 - bytes.len(),
        }));
    }
    let mut found = [0u8; 8];
    found.copy_from_slice(&bytes[..8]);
    if found != magic {
        return Err(LegacyError::BadMagic {
            expected: magic,
            found,
        });
    }
    let v = u16::from_be_bytes([bytes[8], bytes[9]]);
    if v != version {
        return Err(LegacyError::UnsupportedVersion { file, version: v });
    }
    Ok(())
}

impl LegacyDir {
    /// Reads `KDXServer.stg` if the directory has one.
    pub fn load_settings(&self) -> Result<Option<ServerSettings>> {
        self.read(ServerFile::Settings)?
            .map(|b| decode_server_stg(&b))
            .transpose()
    }

    /// Refuses settings whose strings do not fit their length bytes.
    pub fn save_settings(&self, settings: &ServerSettings) -> Result<()> {
        validate_settings(settings)?;
        self.save(ServerFile::Settings, &encode_server_stg(settings)?)
    }
}

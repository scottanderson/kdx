//! 1 Status, the generic reply.

use super::{u32_at, DecodeError};
use crate::MessageType;

/// A 1 Status reply: a request failed, or succeeded with a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The request succeeded.
    Success {
        /// Often the request's type.
        value: u32,
    },
    /// The request failed.
    Error {
        /// Error code, such as 0x10014 Access Denied.
        code: u32,
        /// Type of the failed request.
        request: MessageType,
        /// Source module tag of the server code that raised the error.
        module: u32,
        /// Source line of the server code that raised the error.
        line: u32,
        /// A value the raise site passes along.
        extra: u32,
        /// Bytes 0x14 to 0x17; 0 from the server.
        value_14: u32,
    },
    /// The request failed; Tracker 1.110's short form.
    TrackerError {
        /// Error code, such as 0x10014 Access Denied.
        code: u32,
        /// Type of the failed request.
        request: MessageType,
    },
}

impl Status {
    /// Length of the success form.
    pub const SUCCESS_LEN: usize = 8;
    /// Length of the error form.
    pub const ERROR_LEN: usize = 24;

    /// Length of the tracker's error form.
    pub const TRACKER_ERROR_LEN: usize = 8;

    /// Decodes any form, chosen by length and, at 8 bytes, by whether the first word is 0.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        match buf.len() {
            Self::SUCCESS_LEN if u32_at(buf, 0) == 0 => Ok(Self::Success {
                value: u32_at(buf, 4),
            }),
            Self::TRACKER_ERROR_LEN => Ok(Self::TrackerError {
                code: u32_at(buf, 0),
                request: MessageType(u32_at(buf, 4)),
            }),
            Self::ERROR_LEN => Ok(Self::Error {
                code: u32_at(buf, 0),
                request: MessageType(u32_at(buf, 4)),
                module: u32_at(buf, 8),
                line: u32_at(buf, 0x0c),
                extra: u32_at(buf, 0x10),
                value_14: u32_at(buf, 0x14),
            }),
            _ => Err(DecodeError::Invalid("status length")),
        }
    }

    /// Encodes the reply.
    pub fn encode(&self) -> Vec<u8> {
        let words: &[u32] = match *self {
            Self::Success { value } => &[0, value],
            Self::Error {
                code,
                request,
                module,
                line,
                extra,
                value_14,
            } => &[code, request.0, module, line, extra, value_14],
            Self::TrackerError { code, request } => &[code, request.0],
        };
        words.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    /// The error code, or `None` for success.
    pub const fn code(&self) -> Option<u32> {
        match self {
            Self::Success { .. } => None,
            Self::Error { code, .. } | Self::TrackerError { code, .. } => Some(*code),
        }
    }
}

/// The meaning of an error code, from the server's table of 79 names.
///
/// The table is in `docs/protocol/messages/status.md` under Error codes; a
/// range maps to the class's generic name.
pub const fn error_name(code: u32) -> Option<&'static str> {
    Some(match code {
        0x10001 => "unknown error",
        0x10002 => "invalid value",
        0x10003 => "protocol breach",
        0x10004 => "unimplemented",
        0x10005 => "unsupported",
        0x10006 => "timed out",
        0x10007 => "aborted",
        0x10008 => "data version incompatible",
        0x10009 => "data format incompatible",
        0x1000a => "corrupt data",
        0x1000b => "out of range",
        0x1000c => "item does not exist",
        0x1000d => "item already exists",
        0x1000e => "limit cannot be exceeded",
        0x1000f => "locked",
        0x10010 => "data file missing",
        0x10011 => "resource missing",
        0x10012 => "process does not exist",
        0x10013 => "IPC queue full",
        0x10014 => "Access Denied (a missing privilege)",
        0x10015 => "hardware error",
        0x10016 => "hardware busy",
        0x10017 => "unknown view class",
        0x10018 => "view does not exist",
        0x10019 => "invalid object",
        0x1001a => "buffer too small",
        0x1001b => "unknown OS error",
        0x1001c => "not initialized",
        0x1001d => "re-entry not allowed",
        0x1001e => "invalid name",
        0x20064..=0x20066 => "memory errors",
        0x30064..=0x30065 => "file system errors",
        0x30066 => "file, folder, or item missing",
        0x30067 => "file missing",
        0x30068 => "folder missing",
        0x30069 => "alias missing",
        0x3006a => "expected a file",
        0x3006b => "expected a folder",
        0x3006c => "expected an alias",
        0x3006d => "alias target missing",
        0x3006e => "read-only volume",
        0x3006f => "volume closed",
        0x30070 => "no space",
        0x30071 => "volume limit",
        0x30072 => "wrong passphrase",
        0x30073 => "name already exists",
        0x30074 => "end of file",
        0x30075 => "file is open",
        0x30076 => "invalid name",
        0x30077 => "access denied by the file system",
        0x30078 => "cross-volume operation",
        0x30079 => "folder too deep or alias loop",
        0x3007a => "volume missing",
        0x40064..=0x4006f => "network errors",
        0x40070 => "Incorrect login and/or password.",
        0x40071 => "The network protocol is unknown.",
        0x40072 => "the peer's protocol is too old",
        0x40073 => "the peer's protocol is more recent",
        0x40074 => "Received data that was invalid according to the protocol.",
        0x40075 => "A limit in the net protocol was exceeded.",
        0x40076 => "can't send fast enough",
        0x40077 => "Banned for flooding.",
        0x50064..=0x50066 => "sound errors",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_probe_errors() {
        // 152 Get User Info for a user who doesn't exist.
        let missing = [
            0, 1, 0, 0x0c, 0, 0, 0, 0x98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let status = Status::decode(&missing).unwrap();
        assert_eq!(status.code(), Some(0x1000c));
        assert_eq!(status.encode(), missing);

        // 400 Get File List with a malformed path.
        let corrupt = [
            0, 1, 0, 0x0a, 0, 0, 1, 0x90, 0, 0, 0, 0x46, 0, 0, 0, 0x98, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(
            Status::decode(&corrupt).unwrap(),
            Status::Error {
                code: 0x1000a,
                request: MessageType::GET_FILE_LIST,
                module: 0x46,
                line: 0x98,
                extra: 0,
                value_14: 0,
            }
        );
    }

    #[test]
    fn success() {
        let ok = Status::Success { value: 0x1a5 };
        assert_eq!(Status::decode(&ok.encode()).unwrap(), ok);
    }
}

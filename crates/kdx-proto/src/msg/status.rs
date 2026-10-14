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

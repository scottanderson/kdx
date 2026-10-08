//! File paths, as used by the file messages.

use super::{len_u8, need, u16_at, DecodeError, EncodeError};

/// A path of names below the user's base folder; empty for the root.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilePath(pub Vec<Vec<u8>>);

impl FilePath {
    /// Most names a path may hold.
    pub const MAX_NAMES: usize = 0x400;

    /// A path from names given as text.
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<[u8]>,
    {
        Self(names.into_iter().map(|n| n.as_ref().to_vec()).collect())
    }

    /// Decodes a path that fills `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let (path, len) = Self::decode_prefix(buf)?;
        if len != buf.len() {
            return Err(DecodeError::Invalid("bytes after path"));
        }
        Ok(path)
    }

    /// Decodes a path at the start of `buf` and returns it with its length.
    pub fn decode_prefix(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buf.is_empty() {
            return Ok((Self::default(), 0));
        }
        need(buf, 2)?;
        let count = usize::from(u16_at(buf, 0));
        if count == 0 || count > Self::MAX_NAMES {
            return Err(DecodeError::Invalid("path name count"));
        }
        let mut at = 2;
        let mut names = Vec::with_capacity(count);
        for _ in 0..count {
            need(buf, at + 3)?;
            let len = usize::from(buf[at + 2]);
            if len == 0 {
                return Err(DecodeError::Invalid("empty path name"));
            }
            need(buf, at + 3 + len)?;
            names.push(buf[at + 3..at + 3 + len].to_vec());
            at += 3 + len;
        }
        Ok((Self(names), at))
    }

    /// Encodes the path; the root encodes as nothing.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        if self.0.is_empty() {
            return Ok(Vec::new());
        }
        if self.0.len() > Self::MAX_NAMES {
            return Err(EncodeError::TooLong("path"));
        }
        let mut out = (self.0.len() as u16).to_be_bytes().to_vec();
        for name in &self.0 {
            if name.is_empty() {
                return Err(EncodeError::TooLong("empty path name"));
            }
            out.extend_from_slice(&[0, 0, len_u8(name, "path name")?]);
            out.extend_from_slice(name);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_probe_path() {
        let wire = b"\x00\x01\x00\x00\x0cUploads [UL]";
        let path = FilePath::decode(wire).unwrap();
        assert_eq!(path, FilePath::new(["Uploads [UL]"]));
        assert_eq!(path.encode().unwrap(), wire);
    }

    #[test]
    fn root_and_errors() {
        assert_eq!(FilePath::decode(&[]).unwrap(), FilePath::default());
        assert_eq!(FilePath::default().encode().unwrap(), b"");
        assert!(FilePath::decode(&[0, 0]).is_err());
        assert!(FilePath::decode(&[0, 1, 0, 0, 0]).is_err());
        assert!(FilePath::decode(&[0, 1, 0, 0, 5, b'a']).is_err());
        let two = FilePath::new(["a", "bc"]);
        assert_eq!(FilePath::decode(&two.encode().unwrap()).unwrap(), two);
    }
}

//! The challenge tables that KDX programs embed.
//!
//! Each side of a connection answers the peer's challenge with 20 bytes of its
//! table. A connection starts with [`ChallengeTable::default`], a 31-byte line
//! that every build embeds, and the programs substitute a [`Story`]: the KDX
//! story on every connection, and the GConsole story once a peer says `%HGC`.
//!
//! The stories are Haxial's copyrighted text, so this crate does not contain
//! them. Callers obtain one from a KDX file with [`ChallengeTable::extract`] or
//! load a saved copy with [`ChallengeTable::from_bytes`].

use std::borrow::Cow;

use sha2::{Digest, Sha256};

use crate::handshake::{SUB_PROTOCOL_GCONSOLE, SUB_PROTOCOL_KDX};

/// Length of a challenge response in bytes.
pub const RESPONSE_LEN: usize = 20;

/// The table of a connection that is given none.
pub const DEFAULT: &[u8] = b"All your base are belong to us!";

/// A story that substitutes for the default table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Story {
    /// The 1739-byte story of every client, server, and tracker connection.
    Kdx,
    /// The 3193-byte story, its trailing NUL included, of GConsole connections; only the clients hold it.
    Hgc,
}

impl Story {
    /// Every story.
    pub const ALL: [Self; 2] = [Self::Kdx, Self::Hgc];

    /// The hello sub-protocol of the connections that use the story.
    pub const fn sub_protocol(self) -> [u8; 4] {
        match self {
            Self::Kdx => SUB_PROTOCOL_KDX,
            Self::Hgc => SUB_PROTOCOL_GCONSOLE,
        }
    }

    /// The story's length in bytes.
    pub const fn size(self) -> usize {
        match self {
            Self::Kdx => 1739,
            Self::Hgc => 3193,
        }
    }

    /// Number of valid challenge offsets: `0..offsets()`.
    pub const fn offsets(self) -> u16 {
        (self.size() - RESPONSE_LEN + 1) as u16
    }

    /// Bytes that start the story in every known build.
    pub const fn marker(self) -> &'static [u8] {
        match self {
            Self::Kdx => b"This program including the following story",
            Self::Hgc => b"The following story is Copyright 2004",
        }
    }

    /// SHA-256 of the story.
    pub const fn sha256(self) -> [u8; 32] {
        match self {
            Self::Kdx => [
                0x4f, 0xca, 0x76, 0x36, 0xe7, 0x7e, 0x85, 0xfd, 0xde, 0x95, 0x86, 0x2a, 0x6b, 0xe6,
                0x6e, 0x12, 0xe5, 0xe0, 0x31, 0xad, 0x61, 0xad, 0xd6, 0x5c, 0x5e, 0x97, 0x5d, 0x3d,
                0xaf, 0xf4, 0xb9, 0xe7,
            ],
            Self::Hgc => [
                0xb4, 0x3f, 0x22, 0x22, 0x29, 0x71, 0xa5, 0x7a, 0x0d, 0x8b, 0xdb, 0x3a, 0x5a, 0xf2,
                0x74, 0x7c, 0x7f, 0xf5, 0x11, 0xea, 0x88, 0xf0, 0xba, 0x8c, 0x24, 0xe3, 0x8e, 0x9e,
                0xdc, 0x02, 0x9c, 0x36,
            ],
        }
    }
}

impl std::fmt::Display for Story {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Kdx => "KDX story",
            Self::Hgc => "GConsole story",
        })
    }
}

/// Errors from loading a story.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChallengeError {
    /// The input does not contain the story's starting bytes.
    #[error("no {0} found")]
    NotFound(Story),
    /// The input is not exactly as long as the story.
    #[error("the {story} must be {} bytes, got {got}", story.size())]
    WrongLength {
        /// The story expected.
        story: Story,
        /// The input's length.
        got: usize,
    },
    /// The bytes are not the known story.
    #[error("the {0}'s checksum does not match")]
    WrongChecksum(Story),
}

/// A connection's challenge table: the default, or a verified [`Story`].
#[derive(Clone, PartialEq, Eq)]
pub struct ChallengeTable {
    story: Option<Story>,
    bytes: Cow<'static, [u8]>,
}

impl Default for ChallengeTable {
    fn default() -> Self {
        Self {
            story: None,
            bytes: Cow::Borrowed(DEFAULT),
        }
    }
}

impl ChallengeTable {
    /// Loads a saved story and verifies its checksum.
    pub fn from_bytes(story: Story, bytes: &[u8]) -> Result<Self, ChallengeError> {
        if bytes.len() != story.size() {
            return Err(ChallengeError::WrongLength {
                story,
                got: bytes.len(),
            });
        }
        if Sha256::digest(bytes)[..] != story.sha256() {
            return Err(ChallengeError::WrongChecksum(story));
        }
        Ok(Self {
            story: Some(story),
            bytes: Cow::Owned(bytes.into()),
        })
    }

    /// Finds and verifies the story inside an uncompressed KDX program file.
    pub fn extract(story: Story, file: &[u8]) -> Result<Self, ChallengeError> {
        let marker = story.marker();
        let start = file
            .windows(marker.len())
            .position(|window| window == marker)
            .ok_or(ChallengeError::NotFound(story))?;
        Self::from_bytes(story, &file[start..(start + story.size()).min(file.len())])
    }

    /// The story the table holds, or `None` for the default.
    pub const fn story(&self) -> Option<Story> {
        self.story
    }

    /// The table bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Number of valid challenge offsets: `0..offsets()`.
    pub fn offsets(&self) -> u16 {
        (self.bytes.len() - RESPONSE_LEN + 1) as u16
    }

    /// The response to a challenge, or `None` if `offset` is out of range.
    pub fn response(&self, offset: u16) -> Option<[u8; RESPONSE_LEN]> {
        let start = usize::from(offset);
        self.bytes.get(start..start + RESPONSE_LEN)?.try_into().ok()
    }
}

impl std::fmt::Debug for ChallengeTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.story {
            Some(story) => write!(f, "ChallengeTable({story:?})"),
            None => f.write_str("ChallengeTable(default)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_input() {
        assert_eq!(
            ChallengeTable::extract(Story::Kdx, b"nothing here"),
            Err(ChallengeError::NotFound(Story::Kdx))
        );
        assert_eq!(
            ChallengeTable::from_bytes(Story::Kdx, &[0; 10]),
            Err(ChallengeError::WrongLength {
                story: Story::Kdx,
                got: 10
            })
        );
        assert_eq!(
            ChallengeTable::from_bytes(Story::Hgc, &[0; 3193]),
            Err(ChallengeError::WrongChecksum(Story::Hgc))
        );
    }

    #[test]
    fn gconsole_story_from_the_linux_client() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../vendor/haxial/KDXClient-Lnx-beta5/extracted/KDXClientBETA5-Lnx/KDXClient.lexe"
        );
        let Ok(file) = std::fs::read(path) else {
            eprintln!("skipping: no {path}");
            return;
        };
        let table = ChallengeTable::extract(Story::Hgc, &file).unwrap();
        assert_eq!(table.as_bytes().last(), Some(&0));
    }

    #[test]
    fn default_is_all_your_base() {
        let table = ChallengeTable::default();
        assert_eq!(table.story(), None);
        assert_eq!(table.as_bytes().len(), 0x1f);
        assert_eq!(table.offsets(), 12);
        assert_eq!(table.response(11), Some(*b"se are belong to us!"));
        assert_eq!(table.response(12), None);
    }

    #[test]
    fn offsets_cover_the_whole_story() {
        for story in Story::ALL {
            assert_eq!(
                usize::from(story.offsets()) - 1 + RESPONSE_LEN,
                story.size()
            );
        }
    }
}

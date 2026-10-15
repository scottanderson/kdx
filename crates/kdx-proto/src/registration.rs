//! The UDP datagram a server sends to register with a tracker.
//!
//! The layout, cipher, and checksum are in `docs/protocol/udp.md`.

use crate::msg::{len_u8, need, pad4, u16_at, u32_at, DecodeError, EncodeError};

/// A registration datagram, decrypted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registration {
    /// Bit 0 is [`Self::FROM_TRACKER`].
    pub flags: u16,
    /// Random words the server picks on its first run; the tracker ignores them.
    pub server_id: [u8; 12],
    /// The server's TCP port; a tracker forwards it unchanged.
    pub server_port: u32,
    /// Users logged in.
    pub users: u16,
    /// The string of length byte 0x33; empty from Server 1.620.
    pub string0: Vec<u8>,
    /// The Login of the server's tracker entry; the tracker skips it.
    pub login: Vec<u8>,
    /// Server name.
    pub name: Vec<u8>,
    /// Server description.
    pub description: Vec<u8>,
    /// The string of length byte 0x37; empty from Server 1.620.
    pub string4: Vec<u8>,
    /// Group names; Server 1.620 sends the entry's Groups field as group 0.
    pub groups: [Vec<u8>; 6],
    /// The bytes after the strings; Server 1.620 doesn't clear its padding.
    pub padding: Vec<u8>,
}

impl Registration {
    /// Size of the fixed part before the strings.
    pub const HEADER_LEN: usize = 0x40;
    /// The magic at 0x00.
    pub const MAGIC: &'static [u8; 4] = b"%KDX";
    /// The only version the tracker accepts.
    pub const VERSION: u16 = 0x10;
    /// Flag bit: the sender is a tracker.
    pub const FROM_TRACKER: u16 = 1;
    /// The checksum field's bytes while the CRC runs.
    const CHECKSUM_PLACEHOLDER: [u8; 4] = [0xca, 0xd3, 0x1e, 0xcc];
    const LENS_AT: usize = 0x33;

    /// Decrypts and checks a datagram as Tracker 1.110 does.
    pub fn decode(datagram: &[u8]) -> Result<Self, DecodeError> {
        need(datagram, Self::HEADER_LEN + 1)?;
        let mut buf = datagram.to_vec();
        crypt(&mut buf);
        if &buf[..4] != Self::MAGIC {
            return Err(DecodeError::Invalid("registration magic"));
        }
        if u16_at(&buf, 4) != Self::VERSION {
            return Err(DecodeError::Invalid("registration version"));
        }
        let lens: [u8; 13] = buf[Self::LENS_AT..Self::HEADER_LEN]
            .try_into()
            .expect("13 bytes");
        if lens.iter().all(|&len| len == 0) {
            return Err(DecodeError::Invalid("registration string lengths"));
        }
        let end = Self::HEADER_LEN + lens.iter().map(|&len| usize::from(len)).sum::<usize>();
        need(&buf, end)?;
        let stored = u32_at(&buf, 8);
        buf[8..0x0c].copy_from_slice(&Self::CHECKSUM_PLACEHOLDER);
        if checksum(&buf) != stored {
            return Err(DecodeError::Invalid("registration checksum"));
        }

        let mut at = Self::HEADER_LEN;
        let mut strings = lens.iter().map(|&len| {
            let s = buf[at..at + usize::from(len)].to_vec();
            at += usize::from(len);
            s
        });
        let mut next = || strings.next().expect("13 lengths");
        Ok(Self {
            flags: u16_at(&buf, 6),
            server_id: buf[0x0c..0x18].try_into().expect("12 bytes"),
            server_port: u32_at(&buf, 0x18),
            users: u16_at(&buf, 0x24),
            string0: next(),
            login: next(),
            name: next(),
            description: next(),
            string4: next(),
            groups: std::array::from_fn(|_| next()),
            padding: buf[end..].to_vec(),
        })
    }

    /// Encodes, pads with zeros past [`Self::padding`], checksums, and encrypts the datagram.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let strings = [
            (&self.string0, "registration string 0"),
            (&self.login, "registration login"),
            (&self.name, "server name"),
            (&self.description, "server description"),
            (&self.string4, "registration string 4"),
        ]
        .into_iter()
        .chain(self.groups.iter().map(|g| (g, "group name")));
        let mut out = vec![0; Self::HEADER_LEN];
        out[..4].copy_from_slice(Self::MAGIC);
        out[4..6].copy_from_slice(&Self::VERSION.to_be_bytes());
        out[6..8].copy_from_slice(&self.flags.to_be_bytes());
        out[8..0x0c].copy_from_slice(&Self::CHECKSUM_PLACEHOLDER);
        out[0x0c..0x18].copy_from_slice(&self.server_id);
        out[0x18..0x1c].copy_from_slice(&self.server_port.to_be_bytes());
        out[0x24..0x26].copy_from_slice(&self.users.to_be_bytes());
        for (i, (s, what)) in strings.enumerate() {
            out[Self::LENS_AT + i] = len_u8(s, what)?;
            out.extend_from_slice(s);
        }
        out.extend_from_slice(&self.padding);
        out.resize(pad4(out.len()), 0);
        let sum = checksum(&out);
        out[8..0x0c].copy_from_slice(&sum.to_be_bytes());
        crypt(&mut out);
        Ok(out)
    }
}

/// Applies the registration XOR stream in place; it is its own inverse.
///
/// Bytes past the last whole word are left as they are.
pub fn crypt(buf: &mut [u8]) {
    let mut s: u32 = 0xa5a1_6c4a;
    for word in buf.chunks_exact_mut(4) {
        let value = u32::from_be_bytes([word[0], word[1], word[2], word[3]]) ^ s;
        word.copy_from_slice(&value.to_be_bytes());
        s = s.wrapping_mul(0x41d2_8485).wrapping_add(0x322b);
    }
}

/// CRC-32, polynomial 0x04c11db7 MSB first, initial value 0xdeadbeef, no final XOR.
pub fn checksum(buf: &[u8]) -> u32 {
    buf.iter().fold(0xdead_beef, |crc, &byte| {
        let mut crc = crc ^ (u32::from(byte) << 24);
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04c1_1db7
            } else {
                crc << 1
            };
        }
        crc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Registration {
        Registration {
            server_port: 10700,
            users: 3,
            name: b"Test".to_vec(),
            groups: [
                b"General".to_vec(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
            ..Registration::default()
        }
    }

    #[test]
    fn round_trip() {
        let wire = sample().encode().unwrap();
        assert_eq!(wire.len(), 0x40 + 12);
        let decoded = Registration::decode(&wire).unwrap();
        assert_eq!(decoded.padding, [0]);
        assert_eq!(
            decoded,
            Registration {
                padding: vec![0],
                ..sample()
            }
        );
    }

    #[test]
    fn checks() {
        let wire = sample().encode().unwrap();
        let invalid = |what| Err(DecodeError::Invalid(what));

        assert!(matches!(
            Registration::decode(&wire[..0x40]),
            Err(DecodeError::TooShort { .. })
        ));

        // Flip a plaintext byte, keeping the rest of the stream aligned.
        let flip = |at: usize, bit: u8| {
            let mut wire = wire.clone();
            wire[at] ^= bit;
            Registration::decode(&wire)
        };
        assert_eq!(flip(0, 1), invalid("registration magic"));
        assert_eq!(flip(5, 1), invalid("registration version"));
        assert_eq!(flip(0x40, 1), invalid("registration checksum"));
        assert_eq!(
            flip(0x35, 0x80),
            Err(DecodeError::TooShort {
                need: 0x40 + 0x84 + 7,
                have: 0x4c
            })
        );

        let empty = Registration::default().encode().unwrap();
        assert_eq!(
            Registration::decode(&empty),
            Err(DecodeError::TooShort {
                need: 0x41,
                have: 0x40
            })
        );
        let mut padded = empty;
        padded.extend_from_slice(&[0; 4]);
        assert_eq!(
            Registration::decode(&padded),
            invalid("registration string lengths")
        );
    }
}

//! Byte-level reading and writing of the files' structures. Big-endian
//! throughout, as on the wire; the files embed `kdx-proto` records, which
//! decode from and encode to slices, so the cursor here hands out slices
//! ([`Reader::peek`]) and takes back what they consumed ([`Reader::skip`]).

/// Why a file structure could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// The input ended before the structure did.
    #[error("truncated: needed {needed} more bytes at offset {offset}")]
    Truncated {
        /// Where the read stopped.
        offset: usize,
        /// How many more bytes it wanted.
        needed: usize,
    },
    /// A field holds a value the format does not allow.
    #[error("{0}")]
    Invalid(&'static str),
    /// Bytes were left after the last field.
    #[error("{0} trailing bytes")]
    Trailing(usize),
}

/// Bytes of padding that bring `len` up to a multiple of 4.
pub const fn pad4_len(len: usize) -> usize {
    (4 - len % 4) % 4
}

/// A cursor over a byte slice.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// A cursor at the start of `bytes`.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Reader { bytes, pos: 0 }
    }

    /// Bytes consumed so far.
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Bytes left.
    pub const fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    /// Whether every byte was consumed.
    pub const fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// Everything left, without consuming it; for a `kdx-proto` decoder,
    /// followed by [`skip`](Self::skip) of what it used.
    pub fn peek(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }

    /// Consumes `n` bytes without copying them.
    pub fn skip(&mut self, n: usize) -> Result<(), DecodeError> {
        self.take(n).map(|_| ())
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.remaining() < n {
            return Err(DecodeError::Truncated {
                offset: self.pos,
                needed: n - self.remaining(),
            });
        }
        let out = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    /// One byte.
    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    /// A big-endian `u16`.
    pub fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    /// A big-endian `u32`.
    pub fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    /// A 48-bit big-endian number, as in the header length.
    pub fn u48(&mut self) -> Result<u64, DecodeError> {
        let b = self.take(6)?;
        let mut out = [0u8; 8];
        out[2..].copy_from_slice(b);
        Ok(u64::from_be_bytes(out))
    }

    /// A big-endian `u64`.
    pub fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    /// `N` bytes as an array.
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N)?);
        Ok(out)
    }

    /// `n` bytes, also used for `str` fields, which are raw bytes on the wire.
    pub fn bytes(&mut self, n: usize) -> Result<Vec<u8>, DecodeError> {
        Ok(self.take(n)?.to_vec())
    }

    /// Everything left.
    pub fn rest(&mut self) -> Vec<u8> {
        let out = self.bytes[self.pos..].to_vec();
        self.pos = self.bytes.len();
        out
    }

    /// The `pad4` bytes of a struct that began at `struct_start`: zero to three
    /// bytes up to a multiple of 4. The input must hold them all.
    pub fn pad4(&mut self, struct_start: usize) -> Result<Vec<u8>, DecodeError> {
        let want = pad4_len(self.pos - struct_start);
        self.bytes(want)
    }

    /// Like [`pad4`](Self::pad4), but for a record the originals send either
    /// padded or, at the very end of a payload, not at all (a client's 100 Set
    /// Identity). Returns the empty vector when the input ends exactly here;
    /// a partial padding is still an error. Encoders pair it with
    /// `if !padding.is_empty() { w.pad4(..) }` so both forms re-encode exactly.
    pub fn pad4_lenient(&mut self, struct_start: usize) -> Result<Vec<u8>, DecodeError> {
        if self.is_empty() {
            return Ok(Vec::new());
        }
        self.pad4(struct_start)
    }

    /// Succeeds only if every byte was consumed.
    pub const fn finish(self) -> Result<(), DecodeError> {
        match self.remaining() {
            0 => Ok(()),
            n => Err(DecodeError::Trailing(n)),
        }
    }
}

/// Appends wire fields to a byte vector.
#[derive(Debug, Default, Clone)]
pub struct Writer(Vec<u8>);

impl Writer {
    /// An empty writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bytes written so far.
    pub const fn pos(&self) -> usize {
        self.0.len()
    }

    /// One byte.
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }

    /// A big-endian `u16`.
    pub fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    /// A big-endian `u32`.
    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    /// The low 48 bits, big-endian.
    pub fn u48(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_be_bytes()[2..]);
    }

    /// A big-endian `u64`.
    pub fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    /// Raw bytes.
    pub fn bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }

    /// Writes the padding of a struct that began at `struct_start`: the bytes
    /// kept from decoding, or zeros, up to a multiple of 4.
    pub fn pad4(&mut self, struct_start: usize, kept: &[u8]) {
        let want = pad4_len(self.pos() - struct_start);
        for i in 0..want {
            self.0.push(kept.get(i).copied().unwrap_or(0));
        }
    }

    /// The bytes written.
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u48_reads_six_bytes_big_endian() {
        let mut r = Reader::new(&[0, 1, 0, 0, 0, 2]);
        assert_eq!(r.u48().unwrap(), 0x0001_0000_0002);
        assert!(r.is_empty());
    }

    #[test]
    fn truncated_reports_offset_and_need() {
        let mut r = Reader::new(&[1, 2]);
        assert_eq!(
            r.u32(),
            Err(DecodeError::Truncated {
                offset: 0,
                needed: 2
            })
        );
        assert_eq!(r.pos(), 0);
    }

    #[test]
    fn pad4_from_struct_start() {
        let data = [9, 9, 1, 2, 3, 4, 5, 0xaa, 0xbb, 0xcc, 7];
        let mut r = Reader::new(&data);
        r.u16().unwrap();
        let start = r.pos();
        r.bytes(5).unwrap();
        assert_eq!(r.pad4(start).unwrap(), vec![0xaa, 0xbb, 0xcc]);
        assert_eq!(r.remaining(), 1);
    }

    #[test]
    fn pad4_requires_every_byte() {
        let mut r = Reader::new(&[1, 2, 3, 4, 5, 0xaa]);
        r.bytes(5).unwrap();
        assert_eq!(
            r.pad4(0),
            Err(DecodeError::Truncated {
                offset: 5,
                needed: 2
            })
        );
    }

    #[test]
    fn pad4_lenient_accepts_none_at_the_end_but_not_a_partial() {
        let mut r = Reader::new(&[1, 2, 3, 4, 5]);
        r.bytes(5).unwrap();
        assert_eq!(r.pad4_lenient(0).unwrap(), Vec::<u8>::new());
        let mut r = Reader::new(&[1, 2, 3, 4, 5, 0xaa]);
        r.bytes(5).unwrap();
        assert!(r.pad4_lenient(0).is_err());
        let mut r = Reader::new(&[1, 2, 3, 4, 5, 0xaa, 0xbb, 0xcc, 9]);
        r.bytes(5).unwrap();
        assert_eq!(r.pad4_lenient(0).unwrap(), vec![0xaa, 0xbb, 0xcc]);
    }

    #[test]
    fn finish_reports_trailing() {
        let mut r = Reader::new(&[1, 2, 3]);
        r.u8().unwrap();
        assert_eq!(r.finish(), Err(DecodeError::Trailing(2)));
        assert_eq!(Reader::new(&[]).finish(), Ok(()));
    }

    #[test]
    fn writer_roundtrip_all_widths() {
        let mut w = Writer::new();
        w.u8(1);
        w.u16(0x0203);
        w.u32(0x0405_0607);
        w.u48(0x0809_0a0b_0c0d);
        w.u64(0x0e0f_1011_1213_1415);
        w.bytes(b"ab");
        w.pad4(0, &[0xee]);
        let bytes = w.into_inner();
        assert_eq!(bytes.len(), 24);
        let mut r = Reader::new(&bytes);
        assert_eq!(r.u8().unwrap(), 1);
        assert_eq!(r.u16().unwrap(), 0x0203);
        assert_eq!(r.u32().unwrap(), 0x0405_0607);
        assert_eq!(r.u48().unwrap(), 0x0809_0a0b_0c0d);
        assert_eq!(r.u64().unwrap(), 0x0e0f_1011_1213_1415);
        assert_eq!(r.bytes(2).unwrap(), b"ab");
        assert_eq!(r.pad4(0).unwrap(), vec![0xee]);
        r.finish().unwrap();
    }
}

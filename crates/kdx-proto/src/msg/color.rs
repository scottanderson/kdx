//! A 24-bit color.

/// A color sent as three bytes: red, green, then blue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl Color {
    /// Encoded length.
    pub const LEN: usize = 3;

    /// Decodes the 3 bytes at `at`; the caller checks the length.
    pub(crate) const fn decode_at(buf: &[u8], at: usize) -> Self {
        Self {
            r: buf[at],
            g: buf[at + 1],
            b: buf[at + 2],
        }
    }

    /// Encodes the color.
    pub const fn encode(self) -> [u8; Self::LEN] {
        [self.r, self.g, self.b]
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

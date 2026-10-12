//! Mac OS Roman, the character set of KDX text such as file names.

/// Characters for bytes 0x80 to 0xff; lower bytes are ASCII.
const HIGH: [char; 0x80] = [
    '\u{00c4}', '\u{00c5}', '\u{00c7}', '\u{00c9}', '\u{00d1}', '\u{00d6}', '\u{00dc}', '\u{00e1}',
    '\u{00e0}', '\u{00e2}', '\u{00e4}', '\u{00e3}', '\u{00e5}', '\u{00e7}', '\u{00e9}', '\u{00e8}',
    '\u{00ea}', '\u{00eb}', '\u{00ed}', '\u{00ec}', '\u{00ee}', '\u{00ef}', '\u{00f1}', '\u{00f3}',
    '\u{00f2}', '\u{00f4}', '\u{00f6}', '\u{00f5}', '\u{00fa}', '\u{00f9}', '\u{00fb}', '\u{00fc}',
    '\u{2020}', '\u{00b0}', '\u{00a2}', '\u{00a3}', '\u{00a7}', '\u{2022}', '\u{00b6}', '\u{00df}',
    '\u{00ae}', '\u{00a9}', '\u{2122}', '\u{00b4}', '\u{00a8}', '\u{2260}', '\u{00c6}', '\u{00d8}',
    '\u{221e}', '\u{00b1}', '\u{2264}', '\u{2265}', '\u{00a5}', '\u{00b5}', '\u{2202}', '\u{2211}',
    '\u{220f}', '\u{03c0}', '\u{222b}', '\u{00aa}', '\u{00ba}', '\u{03a9}', '\u{00e6}', '\u{00f8}',
    '\u{00bf}', '\u{00a1}', '\u{00ac}', '\u{221a}', '\u{0192}', '\u{2248}', '\u{2206}', '\u{00ab}',
    '\u{00bb}', '\u{2026}', '\u{00a0}', '\u{00c0}', '\u{00c3}', '\u{00d5}', '\u{0152}', '\u{0153}',
    '\u{2013}', '\u{2014}', '\u{201c}', '\u{201d}', '\u{2018}', '\u{2019}', '\u{00f7}', '\u{25ca}',
    '\u{00ff}', '\u{0178}', '\u{2044}', '\u{20ac}', '\u{2039}', '\u{203a}', '\u{fb01}', '\u{fb02}',
    '\u{2021}', '\u{00b7}', '\u{201a}', '\u{201e}', '\u{2030}', '\u{00c2}', '\u{00ca}', '\u{00c1}',
    '\u{00cb}', '\u{00c8}', '\u{00cd}', '\u{00ce}', '\u{00cf}', '\u{00cc}', '\u{00d3}', '\u{00d4}',
    '\u{f8ff}', '\u{00d2}', '\u{00da}', '\u{00db}', '\u{00d9}', '\u{0131}', '\u{02c6}', '\u{02dc}',
    '\u{00af}', '\u{02d8}', '\u{02d9}', '\u{02da}', '\u{00b8}', '\u{02dd}', '\u{02db}', '\u{02c7}',
];

/// The character for one byte.
pub fn char_from_byte(byte: u8) -> char {
    match byte {
        0..=0x7f => char::from(byte),
        _ => HIGH[usize::from(byte - 0x80)],
    }
}

/// The byte for one character, if Mac OS Roman has it.
pub fn byte_from_char(c: char) -> Option<u8> {
    if c.is_ascii() {
        return Some(c as u8);
    }
    HIGH.iter().position(|&h| h == c).map(|i| 0x80 + i as u8)
}

/// Decodes bytes; every byte has a character, so this is lossless.
pub fn decode(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char_from_byte(b)).collect()
}

/// Encodes text, or returns the first character Mac OS Roman lacks.
pub fn encode(text: &str) -> Result<Vec<u8>, char> {
    text.chars().map(|c| byte_from_char(c).ok_or(c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_byte() {
        let all: Vec<u8> = (0..=0xff).collect();
        assert_eq!(encode(&decode(&all)).unwrap(), all);
    }

    #[test]
    fn known_characters() {
        assert_eq!(decode(b"Erik\xd5s"), "Erik\u{2019}s");
        assert_eq!(encode("Caf\u{e9} \u{20ac}").unwrap(), b"Caf\x8e \xdb");
        assert_eq!(encode("a\u{263a}"), Err('\u{263a}'));
    }
}

/// Applies the handshake XOR stream to `data` in place and returns the final key.
///
/// The stream works on whole big-endian words, so trailing bytes past the last
/// full word are left unchanged.
pub(crate) fn xor_stream(mut key: u32, data: &mut [u8]) -> u32 {
    for word in data.chunks_exact_mut(4) {
        key = (key << 1).wrapping_add(0x4878);
        let value = u32::from_be_bytes([word[0], word[1], word[2], word[3]]) ^ key;
        word.copy_from_slice(&value.to_be_bytes());
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_stream_is_an_involution() {
        let mut data = *b"0123456789abcdef";
        xor_stream(0x1234_5678, &mut data);
        assert_ne!(&data, b"0123456789abcdef");
        xor_stream(0x1234_5678, &mut data);
        assert_eq!(&data, b"0123456789abcdef");
    }
}

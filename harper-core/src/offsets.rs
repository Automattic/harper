/// Build a lookup table that maps every byte offset in `source` to its
/// corresponding character offset.
pub(crate) fn build_byte_to_char_map(source: &str) -> Vec<usize> {
    let mut byte_to_char = vec![0; source.len() + 1];
    let mut char_idx = 0;

    for (byte_idx, ch) in source.char_indices() {
        let next_byte_idx = byte_idx + ch.len_utf8();

        for slot in &mut byte_to_char[byte_idx..next_byte_idx] {
            *slot = char_idx;
        }

        char_idx += 1;
        byte_to_char[next_byte_idx] = char_idx;
    }

    byte_to_char
}

#[cfg(test)]
mod tests {
    use super::build_byte_to_char_map;

    #[test]
    fn maps_ascii_offsets() {
        let map = build_byte_to_char_map("abc");
        assert_eq!(map, vec![0, 1, 2, 3]);
    }

    #[test]
    fn maps_unicode_offsets() {
        let map = build_byte_to_char_map("A🙂B");
        assert_eq!(map, vec![0, 1, 1, 1, 1, 2, 3]);
    }
}

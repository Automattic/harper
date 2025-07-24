//! This module includes various conversions from the index-based [`Span`]s that
//! Harper uses, and the Ranges that the LSP uses.

use harper_core::Span;
use tower_lsp_server::lsp_types::{Position, Range};

pub fn span_to_range(source: &[char], span: Span<char>) -> Range {
    let start = index_to_position(source, span.start);
    let end = index_to_position(source, span.end);

    Range { start, end }
}

fn index_to_position(source: &[char], index: usize) -> Position {
    let before = &source[0..index];
    let newline_indices: Vec<_> = before
        .iter()
        .enumerate()
        .filter_map(|(idx, c)| if *c == '\n' { Some(idx + 1) } else { None })
        .collect();

    let lines = newline_indices.len();

    let last_newline_idx = newline_indices.last().copied().unwrap_or(0);

    let cols: usize = source[last_newline_idx..index]
        .iter()
        .map(|c| c.len_utf16())
        .sum();

    Position {
        line: lines as u32,
        character: cols as u32,
    }
}

/// Converts a position to a (zero-based) character index within the source character array.
///
/// The position is converted to an index using saturating arithmetic. If the requested line index
/// is too high, the index of the last character in the source is returned. If the line is
/// in-bounds but the requested character isn't, the last character of that line is returned.
fn position_to_index(source: &[char], position: Position) -> usize {
    // Find target line.
    let Some(target_line) = source
        // Split including the newline character so we don't lose any characters.
        .split_inclusive(|char| *char == '\n')
        .nth(position.line as usize)
    else {
        // Requested line index is too high.
        // Return the last char in `source' as the closest approximation.
        // Uses `saturating_sub` to avoid underflow when `source` is empty.
        return source.len().saturating_sub(1);
    };

    // Get a pointer to the char we seek.
    // Check if specified character index is within bounds of the target line.
    let target_char_pointer = if position.character
        < target_line
            .len()
            .try_into()
            .expect("target_line.len() can fit in u32")
    {
        // Character index is inside the bounds of the specified line.
        // Calculate pointer to the char we're looking for.
        target_line
            .as_ptr()
            .wrapping_add(position.character as usize)
    } else {
        // Character index is outside the bounds of the specified line.
        // Get pointer to the last character of the line.
        target_line.last().expect("line cannot be empty")
    };

    // Convert the char pointer to its index within `source`.
    // Note: this could be simplified with `offset_from`, but that would require `unsafe`.
    (target_char_pointer as usize - source.as_ptr() as usize) / size_of::<char>()
}

pub fn range_to_span(source: &[char], range: Range) -> Span<char> {
    let start = position_to_index(source, range.start);
    let end = position_to_index(source, range.end);

    Span::new(start, end)
}

#[cfg(test)]
mod tests {
    use tower_lsp_server::lsp_types::{Position, Range};

    use super::{index_to_position, position_to_index, range_to_span};

    #[test]
    fn first_line_correct() {
        let source: Vec<_> = "Hello there.".chars().collect();

        let start = Position {
            line: 0,
            character: 4,
        };

        let i = position_to_index(&source, start);

        assert_eq!(i, 4);

        let p = index_to_position(&source, i);

        assert_eq!(p, start)
    }

    #[test]
    fn reversible_position_conv() {
        let source: Vec<_> = "There was a man,\n his voice had timbre,\n unlike a boy."
            .chars()
            .collect();

        let a = Position {
            line: 1,
            character: 2,
        };

        let b = position_to_index(&source, a);

        assert_eq!(b, 19);

        let c = index_to_position(&source, b);

        let d = position_to_index(&source, a);

        assert_eq!(a, c);
        assert_eq!(b, d);
    }

    #[test]
    fn end_of_line() {
        let source: Vec<_> = "This is a short test\n".chars().collect();

        let a = Position {
            line: 0,
            character: 20,
        };

        assert_eq!(position_to_index(&source, a), 20);
    }

    #[test]
    fn end_of_file() {
        let source: Vec<_> = "This is a short test".chars().collect();

        let a = Position {
            line: 0,
            character: 19,
        };

        assert_eq!(position_to_index(&source, a), 19);
    }

    #[test]
    fn issue_250() {
        let source: Vec<_> = "Hello thur\n".chars().collect();

        let range = Range {
            start: Position {
                line: 0,
                character: 9,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };

        let out = range_to_span(&source, range);
        assert_eq!(out.start, 9);
        assert_eq!(out.end, 10);
    }

    /// Ensures that `position_to_index` does not produce an incorrect index of 0 for an input
    /// `Position` of `{ line: 1, character: 0 }`.
    /// Related to: https://github.com/Automattic/harper/issues/1253
    #[test]
    fn pos_to_index_correct_for_l1_c0() {
        let source: Vec<_> = ". one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty twenty-one twenty-two twenty-three twenty-four twenty-five twenty-six twenty-seven twenty-eight twenty-nine thirty thirty-one\n".chars().collect();
        let position = Position {
            line: 1,
            character: 0,
        };

        let out_index = position_to_index(&source, position);
        assert_ne!(out_index, 0);
    }

    /// Ensures `position_to_index` produces the correct result when indexing line 0 character 0.
    #[test]
    fn pos_to_index_off_by_one_check_l0_c0() {
        let source: Vec<_> = "abc\ndef\nghi\njkl".chars().collect();
        let position = Position {
            line: 0,
            character: 0,
        };

        let out_index = position_to_index(&source, position);
        assert_eq!(source[out_index], 'a');
    }

    /// Ensures `position_to_index` produces the correct result when indexing a non-zero line and
    /// character.
    #[test]
    fn pos_to_index_off_by_one_check_l2_c1() {
        let source: Vec<_> = "abc\ndef\nghi\njkl".chars().collect();
        let position = Position {
            line: 2,
            character: 1,
        };

        let out_index = position_to_index(&source, position);
        assert_eq!(source[out_index], 'h');
    }

    /// Ensures `position_to_index` produces an index of 0 when indexing line 0 character 0 of
    /// a source that contains only a newline (`\n`).
    #[test]
    fn pos_to_index_newline_only_l0_c0() {
        let source: Vec<_> = "\n".chars().collect();
        let position = Position {
            line: 0,
            character: 0,
        };

        let out_index = position_to_index(&source, position);
        assert_eq!(out_index, 0);
    }

    /// Ensures `position_to_index` produces the last character index when indexing an out of
    /// bounds line in a source that contains only newlines (`\n`).
    #[test]
    fn pos_to_index_newlines_only_l7_c0() {
        let source: Vec<_> = "\n\n\n".chars().collect();
        let position = Position {
            line: 7,
            character: 0,
        };

        let out_index = position_to_index(&source, position);
        assert_eq!(out_index, 2);
    }

    /// Ensures `position_to_index` gives the last character of the line when indexing an out of
    /// bounds character.
    #[test]
    fn pos_to_index_out_of_bounds_char() {
        let source: Vec<_> = "abc\ndef\nghi\njkl".chars().collect();
        let position = Position {
            line: 3, // "jkl"
            character: 8,
        };

        let out_index = position_to_index(&source, position);
        assert_eq!(source[out_index], 'l');
    }
}

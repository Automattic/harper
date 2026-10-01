use super::Error;
use crate::CharString;

#[derive(Debug, Clone)]
pub struct AnnotatedWord {
    pub letters: CharString,
    pub annotations: Vec<char>,
}

/// Parse a Rune word list
///
/// Returns [`None`] if the given string is invalid.
pub fn parse_word_list(source: &str) -> Result<Vec<AnnotatedWord>, Error> {
    let mut lines = source.lines().peekable();

    // The curated English dictionary begins with a line holding an approximate
    // item count, which is used to size the output up front. The language module
    // dictionaries have no such line, so it is optional: consume it only when the
    // first line really is a bare integer, and fall back to an unsized `Vec`
    // otherwise.
    let approx_item_count = lines
        .peek()
        .and_then(|first| first.trim().parse::<usize>().ok());

    if approx_item_count.is_some() {
        lines.next();
    }

    let mut words = match approx_item_count {
        Some(count) => Vec::with_capacity(count),
        None => Vec::new(),
    };

    for line in lines {
        // Ignore blank lines and full line comments.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let entry: &str;
        if let Some((entry_part, _comment_part)) = line.split_once('#') {
            entry = entry_part.trim_end();
        } else {
            entry = line.trim_end();
        }

        let word: &str;
        let attr: Option<&str>;
        if let Some((word_part, attr_part)) = entry.split_once('/') {
            word = word_part;
            attr = Some(attr_part);
        } else {
            word = entry;
            attr = None;
        }

        words.push(AnnotatedWord {
            letters: word.chars().collect(),
            annotations: attr.unwrap_or_default().chars().collect(),
        })
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::super::tests::TEST_WORD_LIST;
    use super::parse_word_list;

    #[test]
    fn can_parse_test_file() {
        let list = parse_word_list(TEST_WORD_LIST).unwrap();

        assert_eq!(list.last().unwrap().annotations.len(), 0);
        assert_eq!(list.len(), 4);
    }

    /// The language module dictionaries omit the leading item count that the
    /// curated English dictionary carries. Both forms must parse identically.
    #[test]
    fn leading_item_count_is_optional() {
        let with_count = parse_word_list("4\nhello\ntry/B\nwork/AB\nblank/").unwrap();
        let without_count = parse_word_list("hello\ntry/B\nwork/AB\nblank/").unwrap();

        assert_eq!(with_count.len(), 4);
        assert_eq!(without_count.len(), 4);

        for (a, b) in with_count.iter().zip(without_count.iter()) {
            assert_eq!(a.letters, b.letters);
            assert_eq!(a.annotations, b.annotations);
        }
    }

    /// A first line that merely *starts* with digits is a word, not a count.
    #[test]
    fn numeric_looking_word_is_not_mistaken_for_a_count() {
        let list = parse_word_list("3D/M\nhello").unwrap();

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].letters.as_slice(), &['3', 'D']);
        assert_eq!(list[0].annotations, vec!['M']);
    }

    /// A bare integer on the first line is consumed as the count, so a
    /// dictionary whose first entry is a plain number loses that entry. The
    /// curated dictionary relies on this, so it is behaviour, not a bug.
    #[test]
    fn bare_integer_first_line_is_consumed_as_count() {
        let list = parse_word_list("42\nhello").unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].letters.as_slice(), &['h', 'e', 'l', 'l', 'o']);
    }
}

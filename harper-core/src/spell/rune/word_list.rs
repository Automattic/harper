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
    let mut lines = source.lines();

    let approx_item_count = lines
        .next()
        .ok_or(Error::MalformedItemCount)?
        .parse()
        .map_err(|_| Error::MalformedItemCount)?;

    let mut words = Vec::with_capacity(approx_item_count);

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
}

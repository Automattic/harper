use super::Error;
use crate::CharString;

#[derive(Debug, Clone)]
pub struct MarkedWord {
    pub letters: CharString,
    pub attributes: Vec<char>,
}

/// Parse a Hunspell word list
///
/// Returns [`None`] if the given string is invalid.
pub fn parse_word_list(source: &str) -> Result<Vec<MarkedWord>, Error> {
    let mut lines = source.lines();

    let approx_item_count = lines
        .next()
        .ok_or(Error::MalformedItemCount)?
        .parse()
        .map_err(|_| Error::MalformedItemCount)?;

    let mut words = Vec::with_capacity(approx_item_count);

    for line in lines {
        // blank lines are allowed in the word list
        if line.is_empty() {
            continue;
        }

        let word: &str;
        let attr: Option<&str>; // = Option::None;

        // check for attributes
        if let Some((word_part, attr_part)) = line.split_once('/') {
            word = word_part;

            // word with attributes, throw away any trailing whitespace and comments
            attr = match attr_part.find(|c: char| char::is_ascii_whitespace(&c)) {
                Some(i) => Some(&attr_part[..i]),
                None => Some(&attr_part),
            };
        } else {
            // word without attributes, throw away any trailing whitespace and comments
            word = match line.find(|c: char| char::is_ascii_whitespace(&c)) {
                Some(i) => &line[..i],
                None => line,
            };
            attr = None;
        }

        words.push(MarkedWord {
            letters: word.chars().collect(),
            attributes: attr.unwrap_or_default().chars().collect(),
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

        assert_eq!(list.last().unwrap().attributes.len(), 0);
        assert_eq!(list.len(), 4);
    }
}

use crate::{CharStringExt, Token, TokenKind};

/// The form of a token that the context ranker reads, or `None` if it reads nothing from the token.
pub fn context_word(token: &Token, source: &[char]) -> Option<String> {
    match token.kind {
        TokenKind::Word(_) => Some(
            token
                .get_ch(source)
                .normalized()
                .to_lower()
                .iter()
                .collect(),
        ),
        TokenKind::Number(_) => Some("<NUM>".to_string()),
        _ => None,
    }
}

/// The words on each side of `tokens[index]`, in reading order.
///
/// Each side holds at most `radius` words and stops at the sentence edge.
pub fn surrounding_words(
    tokens: &[Token],
    source: &[char],
    index: usize,
    radius: usize,
) -> (Vec<String>, Vec<String>) {
    let mut left: Vec<String> = tokens[..index]
        .iter()
        .rev()
        .take_while(|token| !token.kind.is_sentence_terminator())
        .filter_map(|token| context_word(token, source))
        .take(radius)
        .collect();
    left.reverse();

    let right = tokens[index + 1..]
        .iter()
        .take_while(|token| !token.kind.is_sentence_terminator())
        .filter_map(|token| context_word(token, source))
        .take(radius)
        .collect();

    (left, right)
}

#[cfg(test)]
mod tests {
    use super::surrounding_words;
    use crate::parsers::{Parser, PlainEnglish};

    fn around(text: &str, target: &str, radius: usize) -> (Vec<String>, Vec<String>) {
        let source: Vec<char> = text.chars().collect();
        let tokens = PlainEnglish.parse(&source);
        let index = tokens
            .iter()
            .position(|t| t.get_ch(&source).iter().collect::<String>() == target)
            .unwrap();

        surrounding_words(&tokens, &source, index, radius)
    }

    #[test]
    fn reads_both_sides_in_order() {
        let (left, right) = around("one two tree four five", "tree", 5);
        assert_eq!(left, ["one", "two"]);
        assert_eq!(right, ["four", "five"]);
    }

    #[test]
    fn honors_the_radius() {
        let (left, right) = around("a b c d e f g", "d", 2);
        assert_eq!(left, ["b", "c"]);
        assert_eq!(right, ["e", "f"]);
    }

    #[test]
    fn stops_at_the_sentence_edge() {
        let (left, right) = around("Old text here. The fibe is corrupt. New text.", "fibe", 5);
        assert_eq!(left, ["the"]);
        assert_eq!(right, ["is", "corrupt"]);
    }

    #[test]
    fn reads_numbers_as_one_symbol() {
        let (left, _) = around("He has 3 fibe", "fibe", 5);
        assert_eq!(left, ["he", "has", "<NUM>"]);
    }

    #[test]
    fn no_surrounding_text() {
        let (left, right) = around("fibe", "fibe", 5);
        assert!(left.is_empty());
        assert!(right.is_empty());
    }

    #[test]
    fn skips_punctuation_and_lowercases_words() {
        let (left, right) = around("One, TWO; fibe (Four)", "fibe", 5);
        assert_eq!(left, ["one", "two"]);
        assert_eq!(right, ["four"]);
    }

    #[test]
    fn the_radius_counts_only_words() {
        let (left, _) = around("a, b, c, d, fibe", "fibe", 2);
        assert_eq!(left, ["c", "d"]);
    }
}

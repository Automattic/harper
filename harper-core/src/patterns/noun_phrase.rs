use crate::Token;

use super::Pattern;

#[derive(Default)]
pub struct NounPhrase;

impl Pattern for NounPhrase {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> usize {
        let mut cursor = 0;

        loop {
            let Some(tok) = tokens.get(cursor) else {
                return 0;
            };

            if tok.kind.is_adjective() || tok.kind.is_article() {
                let Some(next) = tokens.get(cursor + 1) else {
                    return 0;
                };

                if !next.kind.is_whitespace() {
                    return 0;
                }

                cursor += 2;
                continue;
            }

            if tok.kind.is_noun() {
                return cursor + 1;
            }

            return 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::DocPattern;
    use super::NounPhrase;
    use crate::{Document, Span, patterns::Pattern};

    #[test]
    fn simple_apple() {
        let doc = Document::new_markdown_default_curated("A red apple");
        let matches = NounPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(0, 5)])
    }

    #[test]
    fn complex_apple() {
        let doc = Document::new_markdown_default_curated("A red apple with a long stem");
        let matches = NounPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(0, 5), Span::new(8, 13)])
    }

    #[test]
    fn list_fruit() {
        let doc = Document::new_markdown_default_curated("An apple, a banana and a pear");
        let matches = NounPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(
            matches,
            vec![Span::new(0, 3), Span::new(5, 8), Span::new(11, 14)]
        )
    }

    #[test]
    fn simplest_banana() {
        let doc = Document::new_markdown_default_curated("a banana");
        assert!(NounPhrase.matches(doc.get_tokens(), doc.get_source()) != 0);
    }

    #[test]
    fn food() {
        let doc = Document::new_markdown_default_curated(
            "My favorite foods are pizza, sushi, tacos and burgers.",
        );
        let matches = NounPhrase.find_all_matches_in_doc(&doc);

        dbg!(&matches);

        assert_eq!(
            matches,
            vec![
                Span::new(0, 5),
                Span::new(8, 9),
                Span::new(11, 12),
                Span::new(14, 15),
                Span::new(18, 19)
            ]
        )
    }
}

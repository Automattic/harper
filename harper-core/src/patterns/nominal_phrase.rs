use std::num::NonZeroUsize;

use crate::Token;

use super::Pattern;

#[derive(Default)]
pub struct NominalPhrase;

impl Pattern for NominalPhrase {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> Option<NonZeroUsize> {
        let mut cursor = 0;

        loop {
            let tok = tokens.get(cursor)?;

            if tok.kind.is_adjective() || tok.kind.is_determiner() {
                let next = tokens.get(cursor + 1)?;

                if !next.kind.is_whitespace() {
                    return None;
                }

                cursor += 2;
                continue;
            }

            if tok.kind.is_nominal() {
                return NonZeroUsize::new(cursor + 1);
            }

            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::DocPattern;
    use super::NominalPhrase;
    use crate::{Document, Span, patterns::Pattern};

    #[test]
    fn simple_apple() {
        let doc = Document::new_markdown_default_curated("A red apple");
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(0, 5)])
    }

    #[test]
    fn complex_apple() {
        let doc = Document::new_markdown_default_curated("A red apple with a long stem");
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(0, 5), Span::new(8, 13)])
    }

    #[test]
    fn list_fruit() {
        let doc = Document::new_markdown_default_curated("An apple, a banana and a pear");
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(
            matches,
            vec![Span::new(0, 3), Span::new(5, 8), Span::new(11, 14)]
        )
    }

    #[test]
    fn simplest_banana() {
        let doc = Document::new_markdown_default_curated("a banana");
        assert!(
            NominalPhrase
                .matches(doc.get_tokens(), doc.get_source())
                .is_some()
        );
    }

    #[test]
    fn food() {
        let doc = Document::new_markdown_default_curated(
            "My favorite foods are pizza, sushi, tacos and burgers.",
        );
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

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

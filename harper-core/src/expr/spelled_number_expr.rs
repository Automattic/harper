use crate::expr::LongestMatchOf;
use crate::patterns::{WhitespacePattern, WordSet};
use crate::{Span, Token};

use super::{Expr, SequenceExpr};

/// Matches spelled-out numbers from one to ninety-nine
#[derive(Default)]
pub struct SpelledNumberExpr;

impl Expr for SpelledNumberExpr {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        if tokens.is_empty() {
            return None;
        }

        // The numbers that can be in the 2nd position of a compound number.
        // A subset of the standalone numbers since we can't say "twenty zero" or "twenty eleven"
        // "Zero" and "ten" don't belong: twenty-one ✅ twenty-zero ❌ twenty-ten ❌
        let units = &[
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];

        // These can't make a compound with `tens` but they can stand alone
        let teens = &[
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];

        // These can make a compound with the part_2 standalones above.
        // "Ten" and "hundred" don't belong: twenty-one ✅ ten-one ❌ hundred-one ❌
        let tens = &[
            "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];

        let single_words = WordSet::new(
            &units
                .iter()
                .chain(teens.iter())
                .chain(tens.iter())
                .copied()
                .chain(std::iter::once("zero"))
                .collect::<Vec<&str>>(),
        );

        let tens_units_compounds = SequenceExpr::word_set(tens)
            .then_any_of(vec![
                Box::new(|t: &Token, _s: &[char]| t.kind.is_hyphen()),
                Box::new(WhitespacePattern),
            ])
            .then_word_set(units);

        let expr =
            LongestMatchOf::new(vec![Box::new(single_words), Box::new(tens_units_compounds)]);

        expr.run(cursor, tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use super::SpelledNumberExpr;
    use crate::Document;
    use crate::expr::ExprExt;
    use crate::linting::tests::SpanVecExt;

    #[test]
    fn matches_single_digit() {
        let doc = Document::new_markdown_default_curated("one two three");
        let matches = SpelledNumberExpr.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 3);
    }

    #[test]
    fn matches_teens() {
        let doc = Document::new_markdown_default_curated("ten eleven twelve");
        let matches = SpelledNumberExpr.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 3);
    }

    #[test]
    fn matches_tens() {
        let doc = Document::new_markdown_default_curated("twenty thirty forty");
        let matches = SpelledNumberExpr.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 3);
    }

    #[test]
    fn matches_compound_numbers() {
        let doc = Document::new_markdown_default_curated("twenty-one thirty-two");
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        // Debug output
        println!("Found {} matches:", matches.len());
        for m in &matches {
            let text: String = doc.get_tokens()[m.start..m.end]
                .iter()
                .map(|t| doc.get_span_content_str(&t.span))
                .collect();
            println!("- '{text}' (span: {m:?})");
        }

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn deep_thought() {
        let doc = Document::new_markdown_default_curated(
            "the answer to the ultimate question of life, the universe, and everything is forty-two",
        );
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        dbg!(&matches);
        dbg!(matches.to_strings(&doc));

        assert_eq!(matches.to_strings(&doc), vec!["forty-two"]);
    }

    #[test]
    fn jacksons() {
        let doc = Document::new_markdown_default_curated(
            "A, B, C It's easy as one, two, three. Or simple as Do-Re-Mi",
        );
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        assert_eq!(matches.to_strings(&doc), vec!["one", "two", "three"]);
    }

    #[test]
    fn orwell() {
        let doc = Document::new_markdown_default_curated("Nineteen Eighty-Four");
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        assert_eq!(matches.to_strings(&doc), vec!["Nineteen", "Eighty-Four"]);
    }

    #[test]
    fn get_smart() {
        let doc = Document::new_markdown_default_curated(
            "Maxwell Smart was Agent Eighty-Six, but who was Agent Ninety-Nine?",
        );
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        assert_eq!(matches.to_strings(&doc), vec!["Eighty-Six", "Ninety-Nine"]);
    }

    #[test]
    fn hyphens_or_spaces() {
        let doc = Document::new_markdown_default_curated(
            "twenty-one, thirty two, forty-three, fifty four, sixty-five, seventy six, eighty-seven, ninety eight",
        );
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        assert_eq!(
            matches.to_strings(&doc),
            vec![
                "twenty-one",
                "thirty two",
                "forty-three",
                "fifty four",
                "sixty-five",
                "seventy six",
                "eighty-seven",
                "ninety eight",
            ]
        );
    }

    #[test]
    fn waiting_since() {
        let doc = Document::new_markdown_default_curated("I have been waiting since two hours.");
        let matches = SpelledNumberExpr
            .iter_matches_in_doc(&doc)
            .collect::<Vec<_>>();

        assert_eq!(matches.to_strings(&doc), vec!["two"]);
    }
}

use crate::patterns::{IndefiniteArticle, WordSet};
use crate::{Span, Token};

use super::{Expr, SequenceExpr, SpelledNumberExpr};

#[derive(Default)]
pub struct DurationExpr;

impl Expr for DurationExpr {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        if tokens.is_empty() {
            return None;
        }

        let units = WordSet::new(&[
            "minute", "minutes", "hour", "hours", "day", "days", "week", "weeks", "month",
            "months", "year", "years",
        ]);

        let expr = SequenceExpr::longest_of(vec![
            Box::new(SpelledNumberExpr),
            Box::new(SequenceExpr::default().then_number()),
            Box::new(IndefiniteArticle::default()),
        ])
        .then_whitespace()
        .then(units);

        expr.run(cursor, tokens, source)
    }
}

#[cfg(test)]
pub mod tests {
    use super::DurationExpr;
    use crate::Document;
    use crate::expr::ExprExt;
    use crate::linting::tests::SpanVecExt;

    #[test]
    fn detect_10_days() {
        let doc = Document::new_markdown_default_curated("Is 10 days a long time?");
        let matches = DurationExpr.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        assert_eq!(matches.to_strings(&doc), vec!["10 days"]);
    }

    #[test]
    fn detect_ten_days() {
        let doc = Document::new_markdown_default_curated("I think ten days is a long time.");
        let matches = DurationExpr.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        assert_eq!(matches.to_strings(&doc), vec!["ten days"]);
    }
}

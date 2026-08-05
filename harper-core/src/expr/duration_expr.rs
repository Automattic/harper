use crate::{
    Span, Token,
    patterns::{IndefiniteArticle, WordSet},
};

use super::{Expr, SequenceExpr, SpelledNumberExpr};

#[derive(Default)]
pub struct DurationExpr;

impl Expr for DurationExpr {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        if tokens.is_empty() {
            return None;
        }

        let units = WordSet::new(&[
            "second", "seconds", "minute", "minutes", "hour", "hours", "day", "days", "week",
            "weeks", "month", "months", "year", "years",
        ]);

        // standard "a couple of" and colloquial "a couple"
        let a_couple_of = SequenceExpr::default()
            .t_aco("a")
            .t_ws()
            .t_aco("couple")
            .then_optional(SequenceExpr::default().t_ws().t_aco("of"));

        // positive "a few" and negative "few"
        let a_few = SequenceExpr::optional(SequenceExpr::aco("a").t_ws()).t_aco("few");

        let expr = SequenceExpr::longest_of([
            Box::new(SpelledNumberExpr) as Box<dyn Expr>,
            Box::new(SequenceExpr::number()),
            Box::new(IndefiniteArticle::default()),
            Box::new(a_couple_of),
            Box::new(a_few),
            Box::new(SequenceExpr::default().then_quantifier()),
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

    #[test]
    fn detect_a_few_months() {
        let doc = Document::new_markdown_default_curated(
            "I am struggling since a few months with the rebuild of an old FORTRAN program.",
        );
        let matches = DurationExpr.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        assert_eq!(matches.to_strings(&doc), vec!["a few months"]);
    }
}

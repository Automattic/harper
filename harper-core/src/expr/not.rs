use crate::{Span, Token};

use super::Expr;

/// A zero-width assertion that matches when its inner expression does not.
pub struct Not {
    inner: Box<dyn Expr>,
}

impl Not {
    pub fn new(inner: impl Expr + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl Expr for Not {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        self.inner
            .run(cursor, tokens, source)
            .is_none()
            .then(|| Span::empty(cursor))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document,
        expr::{AnchorStart, ExprExt, SequenceExpr},
        linting::tests::SpanVecExt,
    };

    use super::Not;

    #[test]
    fn rejects_expression_at_start() {
        let document = Document::new_plain_english_curated("Give the rise to power.");
        let expression = SequenceExpr::with(Not::new(AnchorStart))
            .then_any_capitalization_of("give")
            .then_whitespace()
            .then_any_capitalization_of("the")
            .then_whitespace()
            .then_any_capitalization_of("rise")
            .then_whitespace()
            .then_any_capitalization_of("to");

        let matches = expression
            .iter_matches_in_doc(&document)
            .collect::<Vec<_>>();

        assert!(matches.is_empty());
    }

    #[test]
    fn matches_expression_after_start() {
        let document = Document::new_plain_english_curated("They give the rise to power.");
        let expression = SequenceExpr::with(Not::new(AnchorStart))
            .then_any_capitalization_of("give")
            .then_whitespace()
            .then_any_capitalization_of("the")
            .then_whitespace()
            .then_any_capitalization_of("rise")
            .then_whitespace()
            .then_any_capitalization_of("to");

        let matches = expression
            .iter_matches_in_doc(&document)
            .collect::<Vec<_>>();

        assert_eq!(matches.to_strings(&document), ["give the rise to"]);
    }
}

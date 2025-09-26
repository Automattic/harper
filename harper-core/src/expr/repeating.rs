use super::Expr;
use crate::{Span, Token};

/// An expression that will match one or more repetitions of the same expression.
///
/// Somewhat reminiscent of the `+*` operator in Regex.
pub struct Repeating {
    inner: Box<dyn Expr>,
    required_repetitions: usize,
}

impl Repeating {
    pub fn new(expr: Box<dyn Expr>, required_repetitions: usize) -> Self {
        Self {
            inner: expr,
            required_repetitions,
        }
    }
}

impl Expr for Repeating {
    fn run(&self, mut cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let mut window = Span::new_with_len(cursor, 0);
        let mut repetition = 0;

        loop {
            let res = self.inner.run(cursor, tokens, source);

            if let Some(res) = res {
                window.expand_to_include(res.start);
                window.expand_to_include(res.end - 1);

                if res.start < cursor {
                    cursor = res.start;
                } else {
                    cursor = res.end;
                }

                if res.is_empty() {
                    return Some(window);
                }

                repetition += 1;
            } else if repetition >= self.required_repetitions {
                return Some(window);
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Repeating;
    use crate::expr::{ExprExt, SequenceExpr};
    use crate::patterns::AnyPattern;
    use crate::{Document, Span};

    #[test]
    fn matches_anything() {
        let doc = Document::new_plain_english_curated(
            "This matcher will match the entirety of any document!",
        );
        let pat = Repeating::new(Box::new(SequenceExpr::from(AnyPattern)), 0);

        assert_eq!(
            pat.iter_matches(doc.get_tokens(), doc.get_source()).next(),
            Some(Span::new(0, doc.get_tokens().len()))
        )
    }

    #[test]
    fn does_not_match_short() {
        let doc = Document::new_plain_english_curated("No match");
        let pat = Repeating::new(Box::new(SequenceExpr::from(AnyPattern)), 4);

        assert_eq!(
            pat.iter_matches(doc.get_tokens(), doc.get_source()).next(),
            None
        )
    }
}

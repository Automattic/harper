use crate::{Span, Token};

use super::Expr;

/// An expression that wraps other expressions to build a filter-line pipeline.
///
/// For example, let's say you wanted to build an expression that matches the spaces between two
/// specific words.
/// To do this, you could start with expression A that detects the pattern `<WORD> <WORD>`. That is,
/// a word, followed by a space, followed by a second word. You could then build Expression B, that
/// simply matches the space. By combining these using a filter, you end up building an expression
/// that matches expression A first, then narrows the result further to only match the resulting
/// space.
///
/// ``` rust
/// use harper_core::patterns::WhitespacePattern;
/// use harper_core::expr::{SequenceExpr, Filter, ExprExt};
/// use harper_core::{Span, Document};
///
/// let a = SequenceExpr::aco("chock").t_ws().t_aco("full");
/// let b = WhitespacePattern;
///
/// let filter = Filter::new(vec![Box::new(a), Box::new(b)]);
/// let doc = Document::new_markdown_default_curated("This test is chock full of insights.");
///
/// let matches: Vec<_> = filter.iter_matches_in_doc(&doc).collect();
/// assert_eq!(vec![Span::new(7, 8)], matches)
/// ```
pub struct Filter {
    steps: Vec<Box<dyn Expr>>,
}

impl Filter {
    pub fn new(steps: Vec<Box<dyn Expr>>) -> Self {
        Self { steps }
    }
}

impl Expr for Filter {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let mut result = self.steps.first()?.run(cursor, tokens, source)?;

        for step in self.steps.iter().skip(1) {
            for i in 0..result.len() {
                let step_res = step.run(i, result.get_content(tokens), source);

                if let Some(step) = step_res {
                    result = step.pushed_by(result.start);
                    break;
                }
            }
        }

        Some(result)
    }
}

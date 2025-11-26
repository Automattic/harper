use std::sync::Arc;

use harper_brill::UPOS;

use crate::char_string::char_string;
use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::patterns::UPOSSet;
use crate::{CharString, Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct AWhile {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<(CharString, &'static str)>>,
}

impl Default for AWhile {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let a = SequenceExpr::default()
            .then(UPOSSet::new(&[UPOS::VERB]))
            .t_ws()
            .t_aco("a")
            .t_ws()
            .t_aco("while");

        map.insert(
            a,
            (
                char_string!("awhile"),
                "Use the single word `awhile` when it follows a verb.",
            ),
        );

        let b = SequenceExpr::default()
            .then_unless(UPOSSet::new(&[UPOS::VERB]))
            .t_ws()
            .t_aco("awhile");

        map.insert(
            b,
            (
                char_string!("a while"),
                "When not used after a verb, spell this duration as `a while`.",
            ),
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for AWhile {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let &(ref suggestion, message) = self.map.lookup(0, matched_tokens, source)?;

        Some(Lint {
            span: matched_tokens[2..].span()?,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::ReplaceWith(suggestion.to_vec())],
            message: message.to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Enforces `awhile` after verbs and `a while` everywhere else."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AWhile;

    #[test]
    fn allow_issue_2144() {
        assert_no_lints(
            "After thinking awhile, I decided to foo a bar.",
            AWhile::default(),
        );
        assert_no_lints(
            "After thinking for a while, I decided to foo a bar.",
            AWhile::default(),
        );
    }

    #[test]
    fn fix_issue_2144() {
        assert_suggestion_result(
            "After thinking a while, I decided to foo a bar.",
            AWhile::default(),
            "After thinking awhile, I decided to foo a bar.",
        );
    }
}

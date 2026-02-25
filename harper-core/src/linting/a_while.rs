use std::sync::Arc;

use harper_brill::UPOS;

use crate::char_string::char_string;
use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::patterns::UPOSSet;
use crate::{CharString, Token, TokenStringExt};

use super::expr_linter::Chunk;
use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct AWhile {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<(CharString, &'static str)>>,
}

impl Default for AWhile {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let a = SequenceExpr::with(UPOSSet::new(&[UPOS::VERB]))
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

        let b = SequenceExpr::unless(UPOSSet::new(&[UPOS::VERB]))
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
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let &(ref suggestion, message) = self.map.lookup(0, matched_tokens, source)?;
        let span = matched_tokens[2..].span()?;
        let suggestion =
            Suggestion::replace_with_match_case(suggestion.to_vec(), span.get_content(source));

        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions: vec![suggestion],
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

    #[test]
    fn correct_in_quite_a_while() {
        assert_suggestion_result(
            "I haven't seen him in quite awhile.",
            AWhile::default(),
            "I haven't seen him in quite a while.",
        );
    }

    #[test]
    fn correct_in_a_while() {
        assert_suggestion_result(
            "I haven't checked in awhile.",
            AWhile::default(),
            "I haven't checked in a while.",
        );
    }

    #[test]
    fn correct_for_awhile() {
        assert_suggestion_result(
            "Video Element Error: MEDA_ERR_DECODE when chrome is left open for awhile",
            AWhile::default(),
            "Video Element Error: MEDA_ERR_DECODE when chrome is left open for a while",
        );
    }

    #[test]
    fn correct_after_awhile() {
        assert_suggestion_result(
            "Links on portal stop working after awhile, requiring page refresh.",
            AWhile::default(),
            "Links on portal stop working after a while, requiring page refresh.",
        );
    }
}

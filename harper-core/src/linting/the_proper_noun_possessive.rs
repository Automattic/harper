use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TheProperNounPossessive {
    expr: Box<dyn Expr>,
}

impl Default for TheProperNounPossessive {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("the")
                    .t_ws()
                    .then(|t: &Token, s: &[char]| {
                        // TODO: should use `k.is_proper_noun()` when #2327 is fixed
                        // TODO: should use `k.is_common_noun()` which doesn't exist yet
                        t.kind.is_possessive_noun()
                            && t.kind.is_titlecase()
                            && !t.kind.is_lowercase()
                            && !t
                                .span
                                .get_content(s)
                                .eq_any_ignore_ascii_case_str(&["internet's", "internet’s"])
                    }),
            ),
        }
    }
}

impl ExprLinter for TheProperNounPossessive {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[..2].span()?,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message:
                "The definite article `the` is redundant before a proper noun in the possessive."
                    .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Checks for redundant `the` before possessive proper noun such as `The London's population`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        tests::{assert_no_lints, assert_suggestion_result},
        the_proper_noun_possessive::TheProperNounPossessive,
    };

    #[test]
    fn fix_the_putins_war() {
        assert_suggestion_result(
            "The Putin's war",
            TheProperNounPossessive::default(),
            "Putin's war",
        );
    }

    #[test]
    fn fix_the_londons_population() {
        assert_suggestion_result(
            "The London's population.",
            TheProperNounPossessive::default(),
            "London's population.",
        )
    }

    #[test]
    fn dont_flag_common_noun_in_titlecase() {
        assert_no_lints("The Dog's Dinner", TheProperNounPossessive::default())
    }

    #[test]
    #[ignore = "Can't currently do this due to issue #???"]
    fn fix_proper_noun_stylized_to_begin_lowercase() {
        assert_suggestion_result(
            "The macOS's Finder",
            TheProperNounPossessive::default(),
            "macOS's Finder",
        );
    }

    #[test]
    fn fix_even_when_capitalisation_omitted() {
        assert_suggestion_result(
            "the egypt's pyramids",
            TheProperNounPossessive::default(),
            "egypt's pyramids",
        )
    }

    #[test]
    fn dont_flag_proper_noun_thats_also_common_noun() {
        assert_no_lints("the china's broken", TheProperNounPossessive::default());
    }

    #[test]
    fn dont_flag_the_internets() {
        assert_no_lints(
            "The internet's most popular icon toolkit has been redesigned",
            TheProperNounPossessive::default(),
        );
    }

    #[test]
    fn dont_flag_the_internets_curly_apostrophe() {
        assert_no_lints(
            "The internet’s most popular icon toolkit has been redesigned",
            TheProperNounPossessive::default(),
        );
    }
}

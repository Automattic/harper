use crate::{
    CharStringExt, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct JealousOf {
    expr: Box<dyn Expr>,
}

impl Default for JealousOf {
    fn default() -> Self {
        let valid_object = |tok: &Token, _source: &[char]| {
            (tok.kind.is_nominal() || tok.kind.is_oov())
                && !tok.kind.is_preposition()
                && !(tok.kind.is_verb() && !tok.kind.is_nominal())
        };

        let pattern = SequenceExpr::default()
            .t_aco("jealous")
            .t_ws()
            .t_aco("from")
            .t_ws()
            .then_optional(SequenceExpr::default().then_determiner().t_ws())
            .then(valid_object);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for JealousOf {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let from_token = tokens.iter().find(|tok| {
            tok.span
                .get_content(source)
                .eq_ignore_ascii_case_str("from")
        })?;

        Some(Lint {
            span: from_token.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of",
                from_token.span.get_content(source),
            )],
            message: "Use `of` after `jealous`.".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Encourages the standard preposition after `jealous`."
    }
}

#[cfg(test)]
mod tests {
    use super::JealousOf;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn replaces_basic_from() {
        assert_suggestion_result(
            "She was jealous from her sister's success.",
            JealousOf::default(),
            "She was jealous of her sister's success.",
        );
    }

    #[test]
    fn handles_optional_determiner() {
        assert_suggestion_result(
            "He grew jealous from the attention.",
            JealousOf::default(),
            "He grew jealous of the attention.",
        );
    }

    #[test]
    fn fixes_pronoun_object() {
        assert_suggestion_result(
            "They became jealous from him.",
            JealousOf::default(),
            "They became jealous of him.",
        );
    }

    #[test]
    fn allows_oov_target() {
        assert_suggestion_result(
            "I'm jealous from Zybrix.",
            JealousOf::default(),
            "I'm jealous of Zybrix.",
        );
    }

    #[test]
    fn corrects_uppercase_preposition() {
        assert_suggestion_result(
            "Jealous FROM his fame.",
            JealousOf::default(),
            "Jealous OF his fame.",
        );
    }

    #[test]
    fn fixes_longer_phrase() {
        assert_suggestion_result(
            "They felt jealous from the sudden praise she received.",
            JealousOf::default(),
            "They felt jealous of the sudden praise she received.",
        );
    }

    #[test]
    fn fixes_minimal_phrase() {
        assert_suggestion_result(
            "jealous from success",
            JealousOf::default(),
            "jealous of success",
        );
    }

    #[test]
    fn does_not_flag_correct_usage() {
        assert_lint_count(
            "She was jealous of her sister's success.",
            JealousOf::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_other_preposition_sequence() {
        assert_lint_count(
            "They stayed jealous from within the fortress.",
            JealousOf::default(),
            0,
        );
    }

    #[test]
    fn fixes_following_gerund() {
        assert_suggestion_result(
            "He was jealous from being ignored.",
            JealousOf::default(),
            "He was jealous of being ignored.",
        );
    }

    #[test]
    fn ignores_numbers_after_from() {
        assert_lint_count(
            "She remained jealous from 2010 through 2015.",
            JealousOf::default(),
            0,
        );
    }
}

use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct BareBones {
    expr: SequenceExpr,
}

impl Default for BareBones {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("bare").t_ws_h().t_aco("bone"),
        }
    }
}

impl ExprLinter for BareBones {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "bones",
                span.get_content(src),
            )],
            message: format!(
                "If this is the idiom meaning `minimal`, it should be plural `bare{}bones`",
                if toks.get(1)?.kind.is_hyphen() {
                    '-'
                } else {
                    ' '
                }
            ),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `bare-bone` to `bare-bones`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::BareBones;

    #[test]
    fn singular_attributive_space() {
        assert_suggestion_result(
            "bare bone csv-parse won't parse anything",
            BareBones::default(),
            "bare bones csv-parse won't parse anything",
        );
    }

    #[test]
    fn singular_attributive_hyphen() {
        assert_suggestion_result(
            "This is a simple bare-bone module which allows background playing from a HTTP stream, using ExoPlayer.",
            BareBones::default(),
            "This is a simple bare-bones module which allows background playing from a HTTP stream, using ExoPlayer.",
        )
    }

    #[test]
    fn dont_flag_plural_attributive_hyphen() {
        assert_no_lints(
            "For the bare-bones version, just display all tags, expanded.",
            BareBones::default(),
        )
    }

    #[test]
    fn dont_flag_plural_attributive_space() {
        assert_no_lints(
            "Support for when JS disabled + a text-only bare bones version",
            BareBones::default(),
        )
    }

    #[test]
    fn dont_flag_plural_predicative_space() {
        assert_no_lints(
            "Even though they are bare bones, they still require some design knowledge to be very successful.",
            BareBones::default(),
        )
    }

    #[test]
    fn dont_flag_plural_predicative_hyphen() {
        assert_no_lints(
            "The core is bare-bones, there are numerous standard protocols since, and many other are in standardization.",
            BareBones::default(),
        )
    }

    #[test]
    fn singular_predicative_space() {
        assert_suggestion_result(
            "It is bare bone because all my views are dinamically created",
            BareBones::default(),
            "It is bare bones because all my views are dinamically created",
        )
    }

    #[test]
    fn singular_predicative_hyphen() {
        assert_suggestion_result(
            "This repo has been created, but it is bare-bone.",
            BareBones::default(),
            "This repo has been created, but it is bare-bone.",
        )
    }
}

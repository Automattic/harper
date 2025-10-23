use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct InterestedIn {
    expr: Box<dyn Expr>,
}

impl Default for InterestedIn {
    fn default() -> Self {
        let pattern = SequenceExpr::default()
            .t_aco("interested")
            .t_ws()
            .then_kind_except(
                TokenKind::is_preposition,
                &["around", "for", "through", "to", "within"],
            );

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for InterestedIn {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let prep_span = tokens.last().unwrap().span;
        let prep_chars = prep_span.get_content(source);

        if prep_chars.eq_ignore_ascii_case_chars(&['i', 'n']) {
            return None;
        }

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(
                "in".chars().collect(),
                prep_chars,
            )],
            message: "The correct preposition to use with `interested` is `in`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Ensures the correct preposition is used with the word `interested` (e.g. `interested in`)."
    }
}

#[cfg(test)]
mod tests {
    use super::InterestedIn;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_about() {
        assert_suggestion_result(
            "It suggests some useful programs the user could be interested about - NyarchLinux/NyarchWizard.",
            InterestedIn::default(),
            "It suggests some useful programs the user could be interested in - NyarchLinux/NyarchWizard.",
        );
    }

    #[test]
    fn dont_flag_around() {
        assert_lint_count(
            "We want to figure out why this is, and how to keep those interested around for longer.",
            InterestedIn::default(),
            0,
        );
    }

    #[test]
    fn fix_at() {
        assert_suggestion_result(
            "If someone is interested at the processed data, please email me.",
            InterestedIn::default(),
            "If someone is interested in the processed data, please email me.",
        );
    }

    #[test]
    #[ignore = "Requires more context because 'interested for now/for sure' are not errors"]
    fn fix_for() {
        assert_suggestion_result(
            "but the user is only interested for the examples in one of the modes",
            InterestedIn::default(),
            "but the user is only interested in the examples in one of the modes",
        );
    }

    #[test]
    fn dont_flag_for_sure() {
        assert_lint_count("I am interested for sure!", InterestedIn::default(), 0);
    }

    #[test]
    fn fix_into() {
        assert_suggestion_result(
            "This is one of the first complex software I wrote, and it prefigures so much of the reasons why I was interested into working on designing HTML and CSS.",
            InterestedIn::default(),
            "This is one of the first complex software I wrote, and it prefigures so much of the reasons why I was interested in working on designing HTML and CSS.",
        );
    }

    #[test]
    fn fix_of() {
        assert_suggestion_result(
            "If you are interested of tinkering.",
            InterestedIn::default(),
            "If you are interested in tinkering.",
        );
    }

    #[test]
    fn fix_on() {
        assert_suggestion_result(
            "The creator of Photopea, a great free alternative to Photoshop, is not interested on making an offline version, so I took it upon myself to make it.",
            InterestedIn::default(),
            "The creator of Photopea, a great free alternative to Photoshop, is not interested in making an offline version, so I took it upon myself to make it.",
        );
    }

    #[test]
    fn dont_flag_through() {
        assert_lint_count(
            "I'm happy to help walk anyone interested through doing this",
            InterestedIn::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_to() {
        assert_lint_count(
            "Hi, As title suggest i am interested to know if we can run a custom model trained on yolov9 inference running on two GPU-s.",
            InterestedIn::default(),
            0,
        );
    }

    #[test]
    fn fix_with() {
        assert_suggestion_result(
            "no_std support (is anybody interested with this?)",
            InterestedIn::default(),
            "no_std support (is anybody interested in this?)",
        );
    }

    #[test]
    fn dont_flag_within() {
        assert_lint_count(
            "But with no one being interested within 8 months, what help would it be.",
            InterestedIn::default(),
            0,
        );
    }
}

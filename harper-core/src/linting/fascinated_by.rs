use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct FascinatedBy {
    expr: Box<dyn Expr>,
}

impl Default for FascinatedBy {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::aco("fascinated").t_ws().then_preposition()),
        }
    }
}

impl ExprLinter for FascinatedBy {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prep_span = toks.last()?.span;
        let prep_chars = prep_span.get_content(src);
        if prep_chars.eq_any_ignore_ascii_case_str(&["by", "with"]) {
            return None;
        }

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("by", prep_chars),
                Suggestion::replace_with_match_case_str("with", prep_chars),
            ],
            message: "The correct prepositions to use with `fascinated` are `by` or `with`."
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Ensures the correct prepositions are used with `fascinated` (e.g., `fascinated by` or `fascinated with`)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{fascinated_by::FascinatedBy, tests::assert_good_and_bad_suggestions};

    #[test]
    fn fix_amiga() {
        assert_good_and_bad_suggestions(
            "Now, one aspect of the Amiga that I've always been fascinated about is making my own games for the Amiga.",
            FascinatedBy::default(),
            &[
                "Now, one aspect of the Amiga that I've always been fascinated by is making my own games for the Amiga.",
                "Now, one aspect of the Amiga that I've always been fascinated with is making my own games for the Amiga.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_microbit() {
        assert_good_and_bad_suggestions(
            "also why I am very fascinated about the micro:bit itself",
            FascinatedBy::default(),
            &[
                "also why I am very fascinated by the micro:bit itself",
                "also why I am very fascinated with the micro:bit itself",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_software_development() {
        assert_good_and_bad_suggestions(
            "Self-learner, fascinated about software development, especially computer graphics and web - marcus-phi.",
            FascinatedBy::default(),
            &[
                "Self-learner, fascinated by software development, especially computer graphics and web - marcus-phi.",
                "Self-learner, fascinated with software development, especially computer graphics and web - marcus-phi.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_computer_science() {
        assert_good_and_bad_suggestions(
            "Fascinated about Computer Science, Finance and Statistics.",
            FascinatedBy::default(),
            &[
                "Fascinated by Computer Science, Finance and Statistics.",
                "Fascinated with Computer Science, Finance and Statistics.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_possibilities() {
        assert_good_and_bad_suggestions(
            "m relatively new to deCONZ and Conbee2 but already very fascinated about the possibilities compared to Philips and Ikea's",
            FascinatedBy::default(),
            &[
                "m relatively new to deCONZ and Conbee2 but already very fascinated by the possibilities compared to Philips and Ikea's",
                "m relatively new to deCONZ and Conbee2 but already very fascinated with the possibilities compared to Philips and Ikea's",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_project() {
        assert_good_and_bad_suggestions(
            "I have been using browser use in local mode for a while and i am pretty fascinated about the project.",
            FascinatedBy::default(),
            &[
                "I have been using browser use in local mode for a while and i am pretty fascinated by the project.",
                "I have been using browser use in local mode for a while and i am pretty fascinated with the project.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_work() {
        assert_good_and_bad_suggestions(
            "Hey guys, I am really fascinated about your work and I tried to build Magisk so I will be able to contribute for the project.",
            FascinatedBy::default(),
            &[
                "Hey guys, I am really fascinated by your work and I tried to build Magisk so I will be able to contribute for the project.",
                "Hey guys, I am really fascinated with your work and I tried to build Magisk so I will be able to contribute for the project.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_ais() {
        assert_good_and_bad_suggestions(
            "I am a retired Dutch telecom engineer and fascinated about AIS applications.",
            FascinatedBy::default(),
            &[
                "I am a retired Dutch telecom engineer and fascinated by AIS applications.",
                "I am a retired Dutch telecom engineer and fascinated with AIS applications.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_innovative_ideas() {
        assert_good_and_bad_suggestions(
            "Software Developer fascinated about innovative ideas, love to learn and share new technologies and ideas.",
            FascinatedBy::default(),
            &[
                "Software Developer fascinated by innovative ideas, love to learn and share new technologies and ideas.",
                "Software Developer fascinated with innovative ideas, love to learn and share new technologies and ideas.",
            ][..],
            &[],
        );
    }

    #[test]
    fn fix_coding() {
        assert_good_and_bad_suggestions(
            "m fascinated about coding and and sharing my code to the world.",
            FascinatedBy::default(),
            &[
                "m fascinated by coding and and sharing my code to the world.",
                "m fascinated with coding and and sharing my code to the world.",
            ][..],
            &[],
        );
    }
}

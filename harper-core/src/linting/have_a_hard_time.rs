use crate::{
    Dialect, Lint, Token,
    expr::{Expr, SequenceExpr},
    indefinite_article::{InitialSound, starts_with_vowel},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct HaveAHardTime {
    expr: SequenceExpr,
    dialect: Dialect,
}

impl HaveAHardTime {
    pub fn new(dialect: Dialect) -> Self {
        Self {
            expr: SequenceExpr::word_set(&["have", "had", "has", "having"])
                .then_optional(SequenceExpr::whitespace().t_set(&["extremely", "real"]))
                .t_ws()
                .then_word_seq(&["hard", "time"]),
            dialect,
        }
    }
}

impl ExprLinter for HaveAHardTime {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let first_space_idx = 1;
        let next_word_idx = first_space_idx + 1;

        let suggestions = match starts_with_vowel(toks[next_word_idx].get_ch(src), self.dialect) {
            Some(InitialSound::Consonant) => &[&['a', ' '][..]][..],
            Some(InitialSound::Vowel) => &[&['a', 'n', ' '][..]][..],
            Some(InitialSound::Either) => &[&['a', ' '][..], &['a', 'n', ' '][..]][..],
            _ => return None,
        };

        Some(Lint {
            span: toks[first_space_idx].span,
            lint_kind: LintKind::Usage,
            suggestions: suggestions
                .iter()
                .map(|s| Suggestion::InsertAfter(s.to_vec()))
                .collect(),
            message: "This idiom requires the indefinite article `a`.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `have hard time` to `have a hard time`."
    }
}

#[cfg(test)]
mod tests {
    use super::HaveAHardTime;
    use crate::{
        Dialect,
        linting::tests::{assert_no_lints, assert_suggestion_result},
    };

    #[test]
    fn had() {
        assert_suggestion_result(
            "I also had hard time to investigate why debug(error) only shows stack traces, but no other visible properties",
            HaveAHardTime::new(Dialect::American),
            "I also had a hard time to investigate why debug(error) only shows stack traces, but no other visible properties",
        );
    }

    #[test]
    fn has() {
        assert_suggestion_result(
            "Typescript has hard time finding declaration file for petite-vue",
            HaveAHardTime::new(Dialect::American),
            "Typescript has a hard time finding declaration file for petite-vue",
        );
    }

    #[test]
    fn have() {
        assert_suggestion_result(
            "I have hard time running getMeasuringContext(text, optionsopt) function.",
            HaveAHardTime::new(Dialect::American),
            "I have a hard time running getMeasuringContext(text, optionsopt) function.",
        );
    }

    #[test]
    fn having() {
        assert_suggestion_result(
            "Instructions for those who are having hard time to install from scratch.",
            HaveAHardTime::new(Dialect::American),
            "Instructions for those who are having a hard time to install from scratch.",
        );
    }

    #[test]
    fn having_extremely() {
        assert_suggestion_result(
            "Bug: having extremely hard time converting html",
            HaveAHardTime::new(Dialect::American),
            "Bug: having an extremely hard time converting html",
        );
    }

    #[test]
    fn having_real() {
        assert_suggestion_result(
            "I am having real hard time integrating kalidokit (face+body tracking)",
            HaveAHardTime::new(Dialect::American),
            "I am having a real hard time integrating kalidokit (face+body tracking)",
        );
    }

    // Oddities and potential false positives

    #[test]
    fn dont_flag_an_hard_time() {
        assert_no_lints(
            "This feature will have an hard time working well with the other ...",
            HaveAHardTime::new(Dialect::American),
        );
    }

    #[test]
    fn dont_flag_some_hard_time() {
        assert_no_lints(
            "we had some hard time finding the right tk because 8.6 has a \"t\" suffix.",
            HaveAHardTime::new(Dialect::American),
        );
    }
}

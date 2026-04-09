use crate::{
    EnglishDialect, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct FedUpWith {
    expr: SequenceExpr,
    dialect: EnglishDialect,
}

impl FedUpWith {
    pub fn new(dialect: EnglishDialect) -> Self {
        let expr = SequenceExpr::fixed_phrase("fed up of");

        Self { expr, dialect }
    }
}

impl ExprLinter for FedUpWith {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if self.dialect == EnglishDialect::British {
            return None;
        }

        let oftok = toks.last()?;
        let ofspan = oftok.span;

        Some(Lint {
            span: ofspan,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "with",
                ofspan.get_content(src),
            )],
            message: "`Fed up of` is not accepted outside of British English.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `fed up of` to `fed up with` in dialects other than British English."
    }
}

#[cfg(test)]
mod tests {
    use super::FedUpWith;
    use crate::EnglishDialect;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn correct_fed_up_of_in_us_english() {
        assert_suggestion_result(
            "I am fed up of Bugzilla reports being ignored.",
            FedUpWith::new(EnglishDialect::American),
            "I am fed up with Bugzilla reports being ignored.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_fed_up_of_in_canadian_english() {
        assert_suggestion_result(
            "Fed up of long links ??? Use ✨ Linsh ✨, a CLI tool to shorten links.",
            FedUpWith::new(EnglishDialect::Canadian),
            "Fed up with long links ??? Use ✨ Linsh ✨, a CLI tool to shorten links.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_fed_up_of_in_aus_english() {
        assert_suggestion_result(
            "Fed up of the lack of Twitter embedded timeline styling options?",
            FedUpWith::new(EnglishDialect::Australian),
            "Fed up with the lack of Twitter embedded timeline styling options?",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_fed_up_of_in_indian_english() {
        assert_suggestion_result(
            "I got fed up of finding my IP (v4) address in the big pile of text that ifconfig outputs on OS X.",
            FedUpWith::new(EnglishDialect::Indian),
            "I got fed up with finding my IP (v4) address in the big pile of text that ifconfig outputs on OS X.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn dont_flag_fed_up_of_in_british_english() {
        assert_no_lints(
            "Fed up of having to repeat the same actions for installing webmin so here's a script for 16.04+",
            FedUpWith::new(EnglishDialect::British),
            crate::languages::LanguageFamily::English,
        );
    }
}

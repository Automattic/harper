use crate::{
    Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
};

const CONTRACTION_AND_POSSESSIVE_ENDINGS: [&str; 7] = ["d", "ll", "m", "re", "s", "t", "ve"];

pub struct WrongApostrophe {
    expr: FirstMatchOf,
}

impl Default for WrongApostrophe {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new(vec![
                Box::new(
                    SequenceExpr::any_word()
                        .then_semicolon()
                        .then_word_set(&CONTRACTION_AND_POSSESSIVE_ENDINGS),
                ),
                Box::new(
                    SequenceExpr::any_word()
                        .then_acute()
                        .then_word_set(&CONTRACTION_AND_POSSESSIVE_ENDINGS),
                ),
            ]),
        }
    }
}

impl ExprLinter for WrongApostrophe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let whole_span = toks.span()?;
        let base = &toks.first()?;
        let ending = &toks.last()?;

        let replacement_str = format!(
            "{}'{}",
            base.get_str(src).to_lowercase(),
            ending.get_str(src).to_lowercase()
        );

        let lettercase_template = [base.get_ch(src), ending.get_ch(src)].concat();

        Some(Lint {
            span: whole_span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case(
                replacement_str.chars().collect(),
                &lettercase_template,
            )],
            message: format!("Did you mean `{replacement_str}`?"),
            priority: 57,
        })
    }

    fn description(&self) -> &str {
        "Corrects semicolons or acute accents typed instead of apostrophes."
    }
}

#[cfg(test)]
mod tests {
    use super::WrongApostrophe;
    use crate::linting::tests::{
        assert_lint_count_with_language, assert_suggestion_result_with_language,
    };

    #[test]
    fn fix_dont_with_semicolon_to_apostrophe() {
        assert_suggestion_result_with_language(
            "It's better if you don;t type like this.",
            WrongApostrophe::default(),
            "It's better if you don't type like this.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn ignore_correct() {
        assert_lint_count_with_language(
            "I don't doubt it.",
            WrongApostrophe::default(),
            0,
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_title_case() {
        assert_suggestion_result_with_language(
            "Don;t type like this.",
            WrongApostrophe::default(),
            "Don't type like this.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_all_caps() {
        assert_suggestion_result_with_language(
            "DON;T TRY THIS AT HOME.",
            WrongApostrophe::default(),
            "DON'T TRY THIS AT HOME.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    #[ignore = "replace_with_match_case has a bug turning `I'll` into `I'LL`"]
    fn fix_ill_and_monkeys() {
        assert_suggestion_result_with_language(
            "Well I;ll be a monkey;s uncle!",
            WrongApostrophe::default(),
            "Well I'll be a monkey's uncle!",
            crate::languages::LanguageFamily::English,
        )
    }

    #[test]
    fn fix_other_contractions_and_possessives() {
        assert_suggestion_result_with_language(
            "Let;s see if we;ve fixed patrakov;s bug. Fun wasn;t it?",
            WrongApostrophe::default(),
            "Let's see if we've fixed patrakov's bug. Fun wasn't it?",
            crate::languages::LanguageFamily::English,
        )
    }

    #[test]
    fn corrects_ive_with_correct_capitalization() {
        assert_suggestion_result_with_language(
            "I;ve",
            WrongApostrophe::default(),
            "I've",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_acute_dont() {
        assert_suggestion_result_with_language(
            "To see the list of available bikes for a location, you don´t need any authentication.",
            WrongApostrophe::default(),
            "To see the list of available bikes for a location, you don't need any authentication.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_acute_im() {
        assert_suggestion_result_with_language(
            "In my research, I´m applying the latest generation of quantitative methods in epidemiology",
            WrongApostrophe::default(),
            "In my research, I'm applying the latest generation of quantitative methods in epidemiology",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_acute_its() {
        assert_suggestion_result_with_language(
            "and it´s auto-updated if that project is hosted here on github",
            WrongApostrophe::default(),
            "and it's auto-updated if that project is hosted here on github",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_acute_lets() {
        assert_suggestion_result_with_language(
            "Let´s now visit the main functionalities provided by GrimoireLab.",
            WrongApostrophe::default(),
            "Let's now visit the main functionalities provided by GrimoireLab.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_acute_microsofts() {
        assert_suggestion_result_with_language(
            "Windows 11 Upgrade tool that bypasses new Microsoft´s requirements",
            WrongApostrophe::default(),
            "Windows 11 Upgrade tool that bypasses new Microsoft's requirements",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn fix_acute_youre() {
        assert_suggestion_result_with_language(
            "You´re looking for clues, but you´re missing all the signs",
            WrongApostrophe::default(),
            "You're looking for clues, but you're missing all the signs",
            crate::languages::LanguageFamily::English,
        );
    }
}

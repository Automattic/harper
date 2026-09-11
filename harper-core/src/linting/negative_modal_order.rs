use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::{ModalVerb, Word},
};

pub struct NegativeModalOrder {
    expr: SequenceExpr,
}

impl Default for NegativeModalOrder {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then(ModalVerb::default())
                .t_ws()
                .then_word_seq(&["have", "not"])
                .t_ws()
                .then_any_of([
                    Box::new(SequenceExpr::default().then_kind_either(
                        TokenKind::is_verb_past_form, // regular verbs: would have not decided
                        TokenKind::is_verb_past_participle_form, // irregular verbs: would have not gone
                    )) as Box<dyn Expr>,
                    Box::new(Word::new("enough")),
                ]),
        }
    }
}

impl ExprLinter for NegativeModalOrder {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let have_sp_not = toks[2..=4].span()?;

        let (have, sp, not) = (
            toks[2].get_ch(src),
            toks[3].get_ch(src),
            toks[4].get_ch(src),
        );

        let not_sp_have: Vec<char> = not
            .iter()
            .chain(sp.iter())
            .chain(have.iter())
            .copied()
            .collect();

        let suggestions = vec![Suggestion::replace_with_match_case(
            not_sp_have,
            have_sp_not.get_content(src),
        )];

        Some(Lint {
            span: have_sp_not,
            lint_kind: LintKind::Usage,
            suggestions,
            message : "Unless you have chosen this word order for emphasis, `not` belongs between the modal verb and `have`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the order of negative modal verb expressions such as `might have not` to `might not have`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::NegativeModalOrder;

    #[test]
    fn may_have_not_verb_past() {
        assert_suggestion_result(
            "Everyone is my child, I may have not birthed them, but I can have a parental feeling to just about anyone",
            NegativeModalOrder::default(),
            "Everyone is my child, I may not have birthed them, but I can have a parental feeling to just about anyone",
        );
    }

    #[test]
    fn may_have_not_enough() {
        assert_suggestion_result(
            "but it may have not enough prioritization to be addressed anytime soon",
            NegativeModalOrder::default(),
            "but it may not have enough prioritization to be addressed anytime soon",
        );
    }

    #[test]
    fn might_have_not_meant() {
        assert_suggestion_result(
            "you might have not meant to close it",
            NegativeModalOrder::default(),
            "you might not have meant to close it",
        );
    }

    #[test]
    fn might_have_not_gotten() {
        assert_suggestion_result(
            "this guy is upset because he might have not gotten the email within 24 hours",
            NegativeModalOrder::default(),
            "this guy is upset because he might not have gotten the email within 24 hours",
        );
    }

    #[test]
    fn must_have_not() {
        assert_suggestion_result(
            "I must have not explained myself very well.",
            NegativeModalOrder::default(),
            "I must not have explained myself very well.",
        );
    }

    #[test]
    fn should_have_not() {
        assert_suggestion_result(
            "should have not shared your token with random ass people",
            NegativeModalOrder::default(),
            "should not have shared your token with random ass people",
        );
    }

    #[test]
    fn would_have_not() {
        assert_suggestion_result(
            "I would have not known that Shelter refuses to uninstall many apps",
            NegativeModalOrder::default(),
            "I would not have known that Shelter refuses to uninstall many apps",
        );
    }

    #[test]
    fn dont_flag_different_sentences() {
        assert_no_lints(
            "Those questions are the same questions I would have. Not sure there's an easy answer",
            NegativeModalOrder::default(),
        );
    }

    #[test]
    fn dont_flag_could_have_not_only() {
        assert_no_lints(
            "Images could have not only alt text, but also fallback images.",
            NegativeModalOrder::default(),
        );
    }

    #[test]
    fn dont_flag_should_have_not_secure() {
        assert_no_lints(
            "It should have NOT SECURE written in big red letters",
            NegativeModalOrder::default(),
        );
    }

    // Known false positives that are not yet handled

    #[test]
    #[ignore = "This is for emphasis, not a grammatical error?"]
    fn dont_flag_could_have_not_used() {
        assert_no_lints(
            "he could have not used the self-type and the ComponentRegistry",
            NegativeModalOrder::default(),
        );
    }

    #[test]
    #[ignore = "For now we look for past participles to avoid false positives."]
    fn doesnt_flag_verb_is_not_past_form() {
        assert_suggestion_result(
            "Victims should have not keep crypto on desktops.",
            NegativeModalOrder::default(),
            "Victims should not have keep crypto on desktops.",
        );
    }

    #[test]
    #[ignore = "This should be flagged after the spello has been fixed."]
    fn doesnt_flag_verb_is_misspelled() {
        assert_suggestion_result(
            "this ad may have not beens loaded or has been disposed",
            NegativeModalOrder::default(),
            "this ad may not have beens loaded or has been disposed",
        );
    }
}

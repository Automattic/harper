use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ForFreeOfCharge {
    expr: SequenceExpr,
}

impl Default for ForFreeOfCharge {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["for", "free", "of", "charge"]).then_any_of(vec![
                Box::new(SequenceExpr::default().then_kind_any(&[
                    TokenKind::is_sentence_terminator,
                    TokenKind::is_comma,
                    TokenKind::is_quote,
                ])),
                Box::new(SequenceExpr::whitespace().then_kind_any_but_not(
                    &[
                        TokenKind::is_conjunction,
                        TokenKind::is_preposition,
                        TokenKind::is_verb,
                    ],
                    TokenKind::is_noun,
                )),
            ]),
        }
    }
}

impl ExprLinter for ForFreeOfCharge {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..7].span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("for free", span.get_content(source)),
                Suggestion::replace_with_match_case_str("free of charge", span.get_content(source)),
            ],
            message: "Use only either `for free` or `free of charge`".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `for free of charge` to either `for free` or `free of charge`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};

    use super::ForFreeOfCharge;

    // TRUE POSITIVES

    #[test]
    fn fix_for_foc_question_mark() {
        // Being followed by a question mark, "for free of charge" is being used as a phrase, not as a modifier
        assert_good_and_bad_suggestions(
            "Where to check my paper grammars for free of charge?",
            ForFreeOfCharge::default(),
            &[
                "Where to check my paper grammars for free?",
                "Where to check my paper grammars free of charge?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_for() {
        // The following `for` is not a noun so can't be modified by "free of charge"
        assert_good_and_bad_suggestions(
            "In Hungary, restaurants are required by law to provide tap water for free of charge for any customers upon request.",
            ForFreeOfCharge::default(),
            &[
                "In Hungary, restaurants are required by law to provide tap water for free for any customers upon request.",
                "In Hungary, restaurants are required by law to provide tap water free of charge for any customers upon request.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_comma_not() {
        // Being followed by a comma, "for free of charge" is being used as a phrase, not as a modifier
        assert_good_and_bad_suggestions(
            "I provide this software in its entirety for free of charge, and support via GitHub is also free.",
            ForFreeOfCharge::default(),
            &[
                "I provide this software in its entirety for free, and support via GitHub is also free.",
                "I provide this software in its entirety free of charge, and support via GitHub is also free.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_quote() {
        // Being within scare quotes, "for free of charge" is being used as a phrase, not as a modifier
        assert_good_and_bad_suggestions(
            "Copyright protects copying, not \"giving things away for free of charge\" or anything like that.",
            ForFreeOfCharge::default(),
            &[
                "Copyright protects copying, not \"giving things away for free\" or anything like that.",
                "Copyright protects copying, not \"giving things away free of charge\" or anything like that.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_otherwise() {
        // `otherwise` is a conjunction that introduces a contrasting clause
        // Therefore, not a noun, meaning `free of charge` is not modifying it
        assert_good_and_bad_suggestions(
            "For Wizzair, if you have purchased a separate ticket for your infant, then only car seat is allowed for free of charge otherwise its not.",
            ForFreeOfCharge::default(),
            &[
                "For Wizzair, if you have purchased a separate ticket for your infant, then only car seat is allowed for free otherwise its not.",
                "For Wizzair, if you have purchased a separate ticket for your infant, then only car seat is allowed free of charge otherwise its not.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_comma_but() {
        assert_good_and_bad_suggestions(
            "user may use the Software for free of charge, but the Software is basically paid software",
            ForFreeOfCharge::default(),
            &[
                "user may use the Software for free, but the Software is basically paid software",
                "user may use the Software free of charge, but the Software is basically paid software",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_period() {
        assert_good_and_bad_suggestions(
            "...giving them away for free of charge. :-) SCNR",
            ForFreeOfCharge::default(),
            &[
                "...giving them away for free. :-) SCNR",
                "...giving them away free of charge. :-) SCNR",
            ],
            &[],
        );
    }

    #[test]
    fn fix_for_foc_are() {
        assert_good_and_bad_suggestions(
            "Also what they offer for free of charge are very very powerful machines for unlimited amount of time.",
            ForFreeOfCharge::default(),
            &[
                "Also what they offer for free are very very powerful machines for unlimited amount of time.",
                "Also what they offer free of charge are very very powerful machines for unlimited amount of time.",
            ],
            &[],
        );
    }

    // FALSE POSITIVES

    #[test]
    fn allows_foc_as_adj_tier() {
        // `free of charge` is an adjective phrase modifying `tier` (and should ideally be hyphenated as "free-of-charge tier")
        assert_no_lints(
            "We prioritize interactive notebook compute for free of charge tier users.",
            ForFreeOfCharge::default(),
        );
    }

    #[test]
    fn allows_foc_as_adj_product() {
        // `free of charge` is an adjective phrase modifying `product` (and should ideally be hyphenated as "free-of-charge product")
        // NOTE: this is probably bad English trying to say "this is the price we pay for using a free product"
        assert_no_lints(
            "This is a cost for free of charge product.",
            ForFreeOfCharge::default(),
        );
    }

    #[test]
    fn allows_foc_as_adj_feedback() {
        // `free of charge` is an adjective phrase modifying `feedback` (and should ideally be hyphenated as "free-of-charge feedback")
        assert_no_lints(
            "... but if you are not a student in a university I don't know where you should turn for free of charge feedback",
            ForFreeOfCharge::default(),
        );
    }

    #[test]
    fn allows_foc_as_adj_place() {
        // `free of charge` is an adjective phrase modifying `place` (and should ideally be hyphenated as "free-of-charge place")
        assert_no_lints(
            "you see how many free places are in a particular garage, so you decide either to occupy it or search for free of charge place somewhere else",
            ForFreeOfCharge::default(),
        );
    }

    #[test]
    fn allows_foc_adj_events() {
        assert_no_lints(
            "Create reservation for free-of-charge events should not require any approval of the reservation",
            ForFreeOfCharge::default(),
        );
    }

    // EDGE CASES NOT YET HANDLED

    #[test]
    fn allows_working_for_free_of_charge() {
        // The `for` is part of `working for`, not part of `for free`
        // NOTE: This probably only passes by fluke since `AnchorEnd` is not in the `Expr`, but `AnchorEnd` is probably
        // NOTE: broken until #3406 is merged
        assert_no_lints(
            "I have a client who I’m working for free of charge",
            ForFreeOfCharge::default(),
        );
    }
}

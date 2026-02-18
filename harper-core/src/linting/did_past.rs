use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct DidPast {
    expr: Box<dyn Expr>,
}

impl Default for DidPast {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["did", "didn't", "didnt"])
                    .then_optional(SequenceExpr::default().t_ws().then_subject_pronoun())
                    .t_ws()
                    // Save effort when the lemma and the simple past form are the same
                    .then_kind_is_but_is_not(
                        TokenKind::is_verb_simple_past_form,
                        TokenKind::is_verb_lemma,
                    ),
            ),
        }
    }
}

impl ExprLinter for DidPast {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Did past"
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let vspan = toks.last()?.span;
        let vchars = vspan.get_content(src);
        let vstr = vspan.get_content_string(src);

        let mut suggestions = vec![];

        // Chop -ed off regular verbs

        if vchars.ends_with_ignore_ascii_case_str("ed") {
            suggestions.push(Suggestion::replace_with_match_case(
                vchars[..vchars.len() - 2].to_vec(),
                vchars,
            ));
        }

        // Look up irregular verbs

        if let Some(lemma) = IrregularVerbs::curated().get_lemma_for_preterite(&vstr) {
            suggestions.push(Suggestion::replace_with_match_case(
                lemma.chars().collect(),
                vchars,
            ));
        }

        if suggestions.len() > 0 {
            Some(Lint {
                span: vspan,
                lint_kind: LintKind::Redundancy,
                suggestions,
                message: "Did past".to_string(),
                ..Default::default()
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DidPast;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // Test basic 'true positive' cases

    #[test]
    fn did_past() {
        assert_suggestion_result("Did went", DidPast::default(), "Did go");
    }

    #[test]
    fn did_past_with_apostrophe() {
        assert_suggestion_result("Didn't saw", DidPast::default(), "Didn't see");
    }

    #[test]
    fn didnt_past_no_apostrophe() {
        assert_suggestion_result("Didnt had", DidPast::default(), "Didnt have");
    }

    #[test]
    fn did_i_heard() {
        assert_suggestion_result("Did I heard", DidPast::default(), "Did I hear");
    }

    #[test]
    fn did_i_heard_with_apostrophe() {
        assert_suggestion_result("Didn't we heard", DidPast::default(), "Didn't we hear");
    }

    #[test]
    fn didnt_i_forgot_no_apostrophe() {
        assert_suggestion_result("Didnt he forgot", DidPast::default(), "Didnt he forget");
    }

    // Test basic 'true negative' cases

    #[test]
    fn ignore_lemma_same_as_past_tense() {
        assert_no_lints("Did read", DidPast::default());
    }

    // Real-world examples

    #[test]
    fn fix_did_you_cmae() {
        assert_suggestion_result(
            "How did you came to this",
            DidPast::default(),
            "How did you come to this",
        );
    }

    #[test]
    fn fix_did_you_wrote() {
        assert_suggestion_result(
            "I'm very interested in the script, if you did wrote it.",
            DidPast::default(),
            "I'm very interested in the script, if you did write it.",
        );
    }

    #[test]
    fn fix_didnt_had() {
        assert_suggestion_result(
            "and i DO know that i didnt had any Terracota",
            DidPast::default(),
            "and i DO know that i didnt have any Terracota",
        );
    }

    #[test]
    fn did_you_went() {
        assert_suggestion_result(
            "Did you went out of memory maybe?",
            DidPast::default(),
            "Did you go out of memory maybe?",
        );
    }

    #[test]
    fn fix_did_needed() {
        assert_suggestion_result(
            "since our CI was broken this did needed to be done",
            DidPast::default(),
            "since our CI was broken this did needed to be done",
        );
    }

    #[test]
    fn fix_did_thought() {
        assert_suggestion_result(
            "I did thought of adding it as a tooltip on hover",
            DidPast::default(),
            "I did think of adding it as a tooltip on hover",
        );
    }

    #[test]
    fn fix_did_wanted() {
        assert_suggestion_result(
            "I did wanted catch all errors in my previous example.",
            DidPast::default(),
            "I did want catch all errors in my previous example.",
        );
    }

    #[test]
    fn ignore_did_you_read() {
        assert_no_lints("Did You Read the Instructions?", DidPast::default());
    }
}

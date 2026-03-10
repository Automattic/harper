use harper_brill::UPOS;

use crate::expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr};
use crate::patterns::UPOSSet;
use crate::{Document, Token, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct DiscourseMarkers {
    expr: SequenceExpr,
}

impl DiscourseMarkers {
    pub fn new() -> Self {
        let phrases = &[
            "however",
            "therefore",
            "meanwhile",
            "furthermore",
            "nevertheless",
            "consequently",
            "thus",
            "instead",
            "moreover",
            "honestly",
            "alternatively",
            "frankly",
            "additionally",
            "subsequently",
            "accordingly",
            "otherwise",
            "incidentally",
            "conversely",
            "notwithstanding",
            "hence",
            "indeed",
            "for example",
            "on the other hand",
        ];

        let phrases_expr = FirstMatchOf::new(
            phrases
                .iter()
                .map(|text: &&str| Box::new(FixedPhrase::from_phrase(text)) as Box<dyn Expr>)
                .collect(),
        );

        Self {
            expr: SequenceExpr::with(phrases_expr)
                .t_ws()
                .then_unless(UPOSSet::new(&[UPOS::ADJ, UPOS::ADV, UPOS::ADP])),
        }
    }

    fn lint_sentence(&self, sent: &[Token], source: &[char]) -> Option<Lint> {
        let first_word_idx = sent.iter_word_indices().next()?;

        if let Some(matched_phrase) = self.expr.run(first_word_idx, sent, source) {
            Some(Lint {
                span: sent[matched_phrase.start..matched_phrase.end - 2].span()?,
                lint_kind: LintKind::Punctuation,
                suggestions: vec![Suggestion::InsertAfter(vec![','])],
                message: "Discourse markers at the beginning of a sentence should be followed by a comma.".into(),
                priority: 31,
            })
        } else {
            None
        }
    }
}

impl Default for DiscourseMarkers {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for DiscourseMarkers {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        document
            .iter_sentences()
            .flat_map(|sent| self.lint_sentence(sent, document.get_source()))
            .collect()
    }

    fn description(&self) -> &str {
        "Flags sentences that begin with a discourse marker but omit the required following comma."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::DiscourseMarkers;

    #[test]
    fn corrects_frankly() {
        assert_suggestion_result(
            "Frankly I think he is wrong.",
            DiscourseMarkers::default(),
            "Frankly, I think he is wrong.",
        );
    }

    #[test]
    fn corrects_however() {
        assert_suggestion_result(
            "However I disagree with your conclusion.",
            DiscourseMarkers::default(),
            "However, I disagree with your conclusion.",
        );
    }

    #[test]
    fn corrects_therefore() {
        assert_suggestion_result(
            "Therefore we must act now.",
            DiscourseMarkers::default(),
            "Therefore, we must act now.",
        );
    }

    #[test]
    fn corrects_meanwhile() {
        assert_suggestion_result(
            "Meanwhile preparations continued in the background.",
            DiscourseMarkers::default(),
            "Meanwhile, preparations continued in the background.",
        );
    }

    #[test]
    fn corrects_furthermore() {
        assert_suggestion_result(
            "Furthermore this approach reduces complexity.",
            DiscourseMarkers::default(),
            "Furthermore, this approach reduces complexity.",
        );
    }

    #[test]
    fn corrects_nevertheless() {
        assert_suggestion_result(
            "Nevertheless we persevered despite the odds.",
            DiscourseMarkers::default(),
            "Nevertheless, we persevered despite the odds.",
        );
    }

    #[test]
    fn corrects_consequently() {
        assert_suggestion_result(
            "Consequently the system halted unexpectedly.",
            DiscourseMarkers::default(),
            "Consequently, the system halted unexpectedly.",
        );
    }

    #[test]
    fn corrects_thus() {
        assert_suggestion_result(
            "Thus we arrive at the final verdict.",
            DiscourseMarkers::default(),
            "Thus, we arrive at the final verdict.",
        );
    }

    #[test]
    fn allows_thus_far() {
        assert_no_lints(
            "Thus far there have been no problems.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_instead() {
        assert_suggestion_result(
            "Instead he chose a different path.",
            DiscourseMarkers::default(),
            "Instead, he chose a different path.",
        );
    }

    #[test]
    fn corrects_moreover() {
        assert_suggestion_result(
            "Moreover this solution is more efficient.",
            DiscourseMarkers::default(),
            "Moreover, this solution is more efficient.",
        );
    }

    #[test]
    fn corrects_alternatively() {
        assert_suggestion_result(
            "Alternatively we could defer the decision.",
            DiscourseMarkers::default(),
            "Alternatively, we could defer the decision.",
        );
    }

    #[test]
    fn no_suggestion_if_comma_present() {
        assert_no_lints(
            "However, I disagree with your point.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn no_lint_for_mid_sentence_marker() {
        assert_no_lints(
            "I said however I would consider it.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn preserves_whitespace() {
        assert_suggestion_result(
            "However   I disagree.",
            DiscourseMarkers::default(),
            "However,   I disagree.",
        );
    }

    #[test]
    fn corrects_semicolon_case() {
        assert_suggestion_result(
            "However I disagree.",
            DiscourseMarkers::default(),
            "However, I disagree.",
        );
    }

    #[test]
    fn corrects_multiple_sentences() {
        assert_suggestion_result(
            "However I disagree. Therefore I propose an alternative.",
            DiscourseMarkers::default(),
            "However, I disagree. Therefore, I propose an alternative.",
        );
    }

    #[test]
    fn allows_single_word_sentence() {
        assert_no_lints("Thus", DiscourseMarkers::default());
    }

    #[test]
    fn corrects_for_example() {
        assert_suggestion_result(
            "For example I recommend updating the configuration.",
            DiscourseMarkers::default(),
            "For example, I recommend updating the configuration.",
        );
    }

    #[test]
    fn no_suggestion_if_comma_after_for_example() {
        assert_no_lints(
            "For example, I recommend updating the configuration.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn preserves_whitespace_for_example() {
        assert_suggestion_result(
            "For example   the outcome was unexpected.",
            DiscourseMarkers::default(),
            "For example,   the outcome was unexpected.",
        );
    }

    #[test]
    fn corrects_on_the_other_hand() {
        assert_suggestion_result(
            "On the other hand we could delay the deployment.",
            DiscourseMarkers::default(),
            "On the other hand, we could delay the deployment.",
        );
    }

    #[test]
    fn no_lint_for_mid_sentence_on_the_other_hand() {
        assert_no_lints(
            "We might postpone, on the other hand this introduces risk.",
            DiscourseMarkers::default(),
        );
    }
}

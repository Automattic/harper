use harper_brill::UPOS;

use crate::expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr};
use crate::patterns::UPOSSet;
use crate::{Document, Token, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

const ALSO_PREPOSITIONS: &[&str] = &["besides", "notwithstanding"];

pub struct DiscourseMarkers {
    expr: SequenceExpr,
}

impl DiscourseMarkers {
    pub fn new() -> Self {
        let phrases = &[
            "accordingly",
            "actually",
            "additionally",
            "also",
            "alternatively",
            "besides",
            "by the way",
            "consequently",
            "conversely",
            "even so",
            "for example",
            "frankly",
            "furthermore",
            "hence",
            "honestly",
            "however",
            "in addition",
            "in other words",
            "incidentally",
            "indeed",
            "instead",
            "meanwhile",
            "moreover",
            "nevertheless",
            "notwithstanding",
            "nowadays",
            "now a days",
            "now-a-days",
            "on the other hand",
            "otherwise",
            "subsequently",
            "that said",
            "therefore",
            "thus",
        ];

        let phrases_expr = FirstMatchOf::new(
            phrases
                .iter()
                .map(|text: &&str| Box::new(FixedPhrase::from_phrase(text)) as Box<dyn Expr>),
        );

        Self {
            // Skip markers followed by a modifier or a verb
            expr: SequenceExpr::with(phrases_expr)
                .t_ws()
                .then_unless(UPOSSet::new(&[
                    UPOS::ADJ,
                    UPOS::ADV,
                    UPOS::ADP,
                    UPOS::CCONJ,
                    UPOS::VERB,
                ])),
        }
    }

    fn lint_sentence(&self, sent: &[Token], source: &[char]) -> Option<Lint> {
        let first_word_idx = sent.iter_word_indices().next()?;
        let matched_phrase = self.expr.run(first_word_idx, sent, source)?;

        let marker_end = matched_phrase.end - 2;
        let marker = &sent[matched_phrase.start..marker_end];

        if self.is_prepositional(marker, &sent[marker_end..], source) {
            return None;
        }

        Some(Lint {
            span: marker.span()?,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::InsertAfter(vec![','])],
            message:
                "Discourse markers at the beginning of a sentence should be followed by a comma."
                    .into(),
            priority: 31,
        })
    }

    fn is_prepositional(&self, marker: &[Token], rest: &[Token], source: &[char]) -> bool {
        let (Some(span), Some(next)) = (marker.span(), rest.first_non_whitespace()) else {
            return false;
        };

        ALSO_PREPOSITIONS.contains(&span.get_content_string(source).to_lowercase().as_str())
            && (next.kind.is_upos(UPOS::DET)
                || !rest
                    .iter()
                    .take_while(|token| !token.kind.is_comma())
                    .any(|token| token.kind.is_upos(UPOS::VERB)))
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

    #[test]
    fn check_2966_is_avoided() {
        assert_no_lints(
            "Honestly and graciously convince someone of something.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_nowadays() {
        assert_suggestion_result(
            "Nowadays E-Commerce is one of the hot topics.",
            DiscourseMarkers::default(),
            "Nowadays, E-Commerce is one of the hot topics.",
        );
    }

    #[test]
    fn no_suggestion_if_nowadays_not_at_start() {
        assert_no_lints(
            "You are the reason the tech state is awful nowadays",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_now_a_days() {
        assert_suggestion_result(
            "Now a days Automation plays a crucial role in mankind.",
            DiscourseMarkers::default(),
            "Now a days, Automation plays a crucial role in mankind.",
        );
    }

    #[test]
    fn no_suggestion_if_comma_after_nowadays() {
        assert_no_lints(
            "Nowadays, data has become the most valuable part of any individual or organization.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn no_suggestion_if_comma_after_now_a_days() {
        assert_no_lints(
            "Now a days, movie recommendation systems are well developed and are user focused.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_also() {
        assert_suggestion_result(
            "Also I think we should go.",
            DiscourseMarkers::default(),
            "Also, I think we should go.",
        );
    }

    #[test]
    fn no_suggestion_if_comma_after_also() {
        assert_no_lints("Also, I think we should go.", DiscourseMarkers::default());
    }

    #[test]
    fn no_lint_for_mid_sentence_also() {
        assert_no_lints("She also said hi.", DiscourseMarkers::default());
    }

    #[test]
    fn allows_also_known_as() {
        assert_no_lints("Also known as the big one.", DiscourseMarkers::default());
    }

    #[test]
    fn allows_also_called() {
        assert_no_lints("Also called the big one.", DiscourseMarkers::default());
    }

    #[test]
    fn allows_also_see() {
        assert_no_lints("Also see chapter two.", DiscourseMarkers::default());
    }

    #[test]
    fn allows_also_available() {
        assert_no_lints("Also available in blue.", DiscourseMarkers::default());
    }

    #[test]
    fn corrects_actually() {
        assert_suggestion_result(
            "Actually I don't know.",
            DiscourseMarkers::default(),
            "Actually, I don't know.",
        );
    }

    #[test]
    fn no_suggestion_if_comma_after_actually() {
        assert_no_lints("Actually, I don't know.", DiscourseMarkers::default());
    }

    #[test]
    fn no_lint_for_mid_sentence_actually() {
        assert_no_lints("She is actually right.", DiscourseMarkers::default());
    }

    #[test]
    fn allows_actually_existing() {
        assert_no_lints(
            "Actually existing socialism was a term.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_also_and_actually_in_one_document() {
        assert_suggestion_result(
            "Actually I don't know. Also I forgot.",
            DiscourseMarkers::default(),
            "Actually, I don't know. Also, I forgot.",
        );
    }

    #[test]
    fn corrects_in_addition() {
        assert_suggestion_result(
            "In addition we need to test this.",
            DiscourseMarkers::default(),
            "In addition, we need to test this.",
        );
    }

    #[test]
    fn allows_in_addition_to() {
        assert_no_lints(
            "In addition to that we need to test this.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_even_so() {
        assert_suggestion_result(
            "Even so the plan failed.",
            DiscourseMarkers::default(),
            "Even so, the plan failed.",
        );
    }

    #[test]
    fn allows_even_so_much_as() {
        assert_no_lints(
            "Even so much as a whisper would wake him.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_that_said() {
        assert_suggestion_result(
            "That said we should wait.",
            DiscourseMarkers::default(),
            "That said, we should wait.",
        );
    }

    #[test]
    fn corrects_in_other_words() {
        assert_suggestion_result(
            "In other words we should go.",
            DiscourseMarkers::default(),
            "In other words, we should go.",
        );
    }

    #[test]
    fn allows_in_other_words_and_phrases() {
        assert_no_lints(
            "In other words and phrases, keep it short.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_by_the_way() {
        assert_suggestion_result(
            "By the way I saw him yesterday.",
            DiscourseMarkers::default(),
            "By the way, I saw him yesterday.",
        );
    }

    #[test]
    fn allows_by_way_of() {
        assert_no_lints(
            "By way of introduction he waved.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_besides() {
        assert_suggestion_result(
            "Besides we have no time.",
            DiscourseMarkers::default(),
            "Besides, we have no time.",
        );
    }

    #[test]
    fn corrects_besides_before_list() {
        assert_suggestion_result(
            "Besides we need milk, eggs, and bread.",
            DiscourseMarkers::default(),
            "Besides, we need milk, eggs, and bread.",
        );
    }

    #[test]
    fn allows_besides_as_preposition() {
        assert_no_lints(
            "Besides me, nobody knew the answer.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_notwithstanding() {
        assert_suggestion_result(
            "Notwithstanding they still went out.",
            DiscourseMarkers::default(),
            "Notwithstanding, they still went out.",
        );
    }

    #[test]
    fn allows_notwithstanding_as_preposition() {
        assert_no_lints(
            "Notwithstanding the rain, we went out.",
            DiscourseMarkers::default(),
        );
    }

    #[test]
    fn corrects_other_markers_before_determiner() {
        assert_suggestion_result(
            "However the rain held off.",
            DiscourseMarkers::default(),
            "However, the rain held off.",
        );
    }
}

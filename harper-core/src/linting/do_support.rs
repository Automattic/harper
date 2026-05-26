//! Lint rule for common "do"-support errors in English.
//!
//! Detects:
//! 1. Redundant "do" before another auxiliary/modal ("He does can swim" → "He can swim")
//! 2. Redundant "do" in affirmative statements before non-emphatic verbs
//!    ("They do have went" → "They have gone")
//! 3. "Do" + past tense verb instead of base form ("They do went" → "They go")
//!
//! Does NOT flag:
//! - Emphatic "do" in statements ("I **do** like it")
//! - "Do" in questions ("Do you like it?")
//! - "Do" as a main verb ("I'll do my homework")

use crate::{
    CharStringExt, Token,
    char_ext::CharExt,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Sentence},
    patterns::{ModalVerb, Pattern},
    spell::Dictionary,
};

/// The main struct for the `DoSupport` linter.
pub struct DoSupport<D> {
    expr: SequenceExpr,
    dict: D,
}

impl<D> DoSupport<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        // Pattern 1: do/does/did followed immediately by a modal
        // e.g. "They do can swim", "He does should go"
        let do_plus_modal =
            SequenceExpr::word_set(&["do", "does", "did", "don't", "doesn't", "didn't"])
                .t_ws()
                .then(ModalVerb::without_common_errors());

        // Pattern 2: do/does/did + past-tense verb (should be base form)
        // e.g. "They do went", "She does came", "He did saw"
        let do_plus_past =
            SequenceExpr::word_set(&["do", "does", "did", "don't", "doesn't", "didn't"])
                .t_ws()
                .then_kind_where(|k| {
                    (k.is_verb_simple_past_form() || k.is_verb_past_form())
                        && !k.is_verb_lemma()
                        && !k.is_verb_past_participle_only()
                });

        Self {
            expr: SequenceExpr::any_of(vec![Box::new(do_plus_modal), Box::new(do_plus_past)]),
            dict,
        }
    }

    /// Try to get the base form (lemma) of a past-tense verb.
    fn get_lemma_for_past(&self, verb_chars: &[char], _src: &[char]) -> Option<Vec<char>> {
        let verb_str: String = verb_chars.iter().collect();

        // First, try the irregular verbs table
        if let Some(lemma) = IrregularVerbs::curated().get_lemma_for_preterite(&verb_str) {
            return Some(lemma.chars().collect());
        }

        // For regular verbs, try chopping -d/-ed
        let mut suggs: Vec<Vec<char>> = Vec::new();

        if verb_chars.ends_with_ignore_ascii_case_chars(&['d']) {
            let without_d = &verb_chars[..verb_chars.len() - 1];

            if without_d.ends_with_ignore_ascii_case_chars(&['e']) {
                let without_ed = &without_d[..without_d.len() - 1];

                if self.is_verb_lemma(without_ed) {
                    suggs.push(without_ed.to_vec());
                }

                // -ied → -y (e.g. "fried" → "fry")
                if without_ed.ends_with_ignore_ascii_case_chars(&['i']) {
                    let mut with_final_y = without_ed[..without_ed.len() - 1].to_vec();
                    with_final_y.push('y');
                    if self.is_verb_lemma(&with_final_y) {
                        suggs.push(with_final_y);
                    }
                }

                // doubled consonant (e.g. "logged" → "log")
                if let Some(last) = without_ed.last() {
                    if !last.is_vowel() && without_ed.len() > 1 {
                        let without_doubled = without_ed[..without_ed.len() - 1].to_vec();
                        if self.is_verb_lemma(&without_doubled) {
                            suggs.push(without_doubled);
                        }
                    }
                }
            }

            if self.is_verb_lemma(without_d) {
                suggs.push(without_d.to_vec());
            }
        }

        suggs.into_iter().next()
    }

    fn is_verb_lemma(&self, chars: &[char]) -> bool {
        self.dict
            .get_word_metadata(chars)
            .is_some_and(|md| md.is_verb_lemma())
    }
}

impl<D> ExprLinter for DoSupport<D>
where
    D: Dictionary,
{
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // First token should be do/does/did
        let do_tok = toks.first()?;
        let do_word = do_tok.span.get_content_string(src);

        // Don't flag "do" as a main verb (e.g. "I'll do my homework")
        // We only flag when the following word is a modal or past-tense verb

        // Check the second token
        let second_tok = toks.get(1)?;
        let second_word = second_tok.span.get_content(src);
        let second_str = second_tok.span.get_content_string(src);

        // If second word is a modal, suggest removing "do"
        let is_modal = ModalVerb::without_common_errors()
            .matches(&toks[1..], src)
            .is_some();

        if is_modal {
            // "They do can swim" → "They can swim"
            return Some(Lint {
                span: do_tok.span,
                lint_kind: LintKind::Redundancy,
                suggestions: vec![Suggestion::Remove],
                message: format!(
                    "The auxiliary \"{}\" is redundant before the modal \"{}\".",
                    do_word, second_str
                ),
                priority: 31,
            });
        }

        // If second word is a past-tense verb, suggest base form
        let is_past =
            second_tok.kind.is_verb_simple_past_form() || second_tok.kind.is_verb_past_form();

        if is_past && !second_tok.kind.is_verb_lemma() {
            // Don't flag in questions — "Did you went?" is handled by DidPast
            // We only flag affirmative statements where "do" is redundant
            // or where "do" + past tense should be "do" + base form

            // Check if this is likely a question by looking at context
            // In questions, "do" is necessary for inversion
            if let Some((before, _)) = context {
                // If the sentence starts with "do/does/did", it's likely a question
                if before.is_empty() || (before.len() == 1 && before[0].kind.is_whitespace()) {
                    // Sentence-initial "do" — likely a question, skip
                    // (DidPast handles the past-tense issue in questions)
                    return None;
                }
            }

            if let Some(lemma_chars) = self.get_lemma_for_past(second_word, src) {
                let lemma_str: String = lemma_chars.iter().collect();
                return Some(Lint {
                    span: second_tok.span,
                    lint_kind: LintKind::Redundancy,
                    suggestions: vec![Suggestion::replace_with_match_case(
                        lemma_chars,
                        second_word,
                    )],
                    message: format!("Use the base form \"{}\" after \"{}\".", lemma_str, do_word),
                    priority: 31,
                });
            }
        }

        None
    }

    fn description(&self) -> &str {
        "Detects redundant \"do\" before modals and corrects past-tense verbs to base form after \"do\" in affirmative statements."
    }
}

#[cfg(test)]
mod tests {
    use super::DoSupport;
    use crate::{
        linting::tests::{assert_no_lints, assert_suggestion_result},
        spell::FstDictionary,
    };

    // ── do + modal: redundant "do" ──────────────────────────────────────

    #[test]
    fn fix_do_can() {
        assert_suggestion_result(
            "He does can swim.",
            DoSupport::new(FstDictionary::curated()),
            "He can swim.",
        );
    }

    #[test]
    fn fix_do_will() {
        assert_suggestion_result(
            "They do will go tomorrow.",
            DoSupport::new(FstDictionary::curated()),
            "They will go tomorrow.",
        );
    }

    #[test]
    fn fix_did_could() {
        assert_suggestion_result(
            "She did could help with the project.",
            DoSupport::new(FstDictionary::curated()),
            "She could help with the project.",
        );
    }

    #[test]
    fn fix_do_should() {
        assert_suggestion_result(
            "We do should review the code.",
            DoSupport::new(FstDictionary::curated()),
            "We should review the code.",
        );
    }

    #[test]
    fn fix_does_must() {
        assert_suggestion_result(
            "It does must be done.",
            DoSupport::new(FstDictionary::curated()),
            "It must be done.",
        );
    }

    #[test]
    fn fix_do_might() {
        assert_suggestion_result(
            "He do might come later.",
            DoSupport::new(FstDictionary::curated()),
            "He might come later.",
        );
    }

    #[test]
    fn fix_do_would() {
        assert_suggestion_result(
            "They do would like to attend.",
            DoSupport::new(FstDictionary::curated()),
            "They would like to attend.",
        );
    }

    // ── do/does/did + past tense: correct to base form ──────────────────

    #[test]
    fn fix_do_went() {
        assert_suggestion_result(
            "They do went to the store.",
            DoSupport::new(FstDictionary::curated()),
            "They do go to the store.",
        );
    }

    #[test]
    fn fix_does_came() {
        assert_suggestion_result(
            "She does came early.",
            DoSupport::new(FstDictionary::curated()),
            "She does come early.",
        );
    }

    #[test]
    fn fix_does_wrote() {
        assert_suggestion_result(
            "He does wrote the report.",
            DoSupport::new(FstDictionary::curated()),
            "He does write the report.",
        );
    }

    #[test]
    fn fix_did_saw_regular() {
        // "saw" as past of "see" → "see"
        assert_suggestion_result(
            "They did saw the problem.",
            DoSupport::new(FstDictionary::curated()),
            "They did see the problem.",
        );
    }

    #[test]
    fn fix_do_took() {
        assert_suggestion_result(
            "She do took the last bus.",
            DoSupport::new(FstDictionary::curated()),
            "She do take the last bus.",
        );
    }

    #[test]
    fn fix_does_gave() {
        assert_suggestion_result(
            "He does gave a presentation.",
            DoSupport::new(FstDictionary::curated()),
            "He does give a presentation.",
        );
    }

    #[test]
    fn fix_did_forgot() {
        assert_suggestion_result(
            "She did forgot her keys.",
            DoSupport::new(FstDictionary::curated()),
            "She did forget her keys.",
        );
    }

    #[test]
    fn fix_does_fried() {
        // -ied → -y pattern
        assert_suggestion_result(
            "He does fried the fish.",
            DoSupport::new(FstDictionary::curated()),
            "He does fry the fish.",
        );
    }

    #[test]
    fn fix_does_logged() {
        // doubled consonant pattern
        assert_suggestion_result(
            "It does logged the error.",
            DoSupport::new(FstDictionary::curated()),
            "It does log the error.",
        );
    }

    // ── No false positives ──────────────────────────────────────────────

    #[test]
    fn no_flag_question_did_you() {
        // "Did you went?" is a question — handled by DidPast, not DoSupport
        assert_no_lints(
            "Did you went to the store?",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_do_as_main_verb() {
        // "do" as a main verb — not followed by modal or past tense
        assert_no_lints(
            "I'll do my homework.",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_emphatic_do() {
        // Emphatic "do" + base form — this is correct
        assert_no_lints("I do like pizza.", DoSupport::new(FstDictionary::curated()));
    }

    #[test]
    fn no_flag_do_have_base() {
        // "do have" where "have" is a base form — correct emphatic usage
        assert_no_lints(
            "They do have a point.",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_question_does_it() {
        // Question starting with "does" — not flagged
        assert_no_lints(
            "Does it worked properly?",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_normal_question() {
        assert_no_lints(
            "Do you need help?",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_do_your_best() {
        // "do" as main verb
        assert_no_lints(
            "Just do your best.",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_things_to_do() {
        assert_no_lints(
            "There are things to do.",
            DoSupport::new(FstDictionary::curated()),
        );
    }
}

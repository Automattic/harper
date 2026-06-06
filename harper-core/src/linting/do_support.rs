//! Lint rule for "do"-support errors in English.
//!
//! Detects:
//! 1. Redundant "do" before another auxiliary/modal ("He does can swim" → "He can swim")
//! 2. "Do" + past tense verb instead of base form ("They do went" → "They do go")
//! 3. Missing "do"-support in wh-questions with object-fronted verbs
//!    ("What means this?" → "What does this mean?")
//!    ("How works it?" → "How does it work?")
//!
//! Does NOT flag:
//! - Emphatic "do" in statements ("I **do** like it")
//! - "Do" in questions ("Do you like it?")
//! - "Do" as a main verb ("I'll do my homework")
//! - Wh-word as subject ("What works best?") — no do-support needed
//! - "Means" as a noun ("By what means did they enter?")

use crate::{
    CharStringExt, Token, TokenStringExt,
    char_ext::CharExt,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Sentence},
    patterns::{ModalVerb, Pattern},
    spell::Dictionary,
};

/// Wh-words that can front an object question.
const WH_WORDS: &[&str] = &[
    "what", "which", "who", "whom", "whose", "where", "when", "why", "how",
];

/// Words that are always auxiliary verbs (never main verbs in questions).
/// When one of these follows a wh-word, do-support is NOT needed
/// because auxiliaries already invert: "What is it?" "Where can we go?"
const AUXILIARIES: &[&str] = &[
    "is",
    "are",
    "was",
    "were",
    "am",
    "can",
    "could",
    "will",
    "would",
    "shall",
    "should",
    "may",
    "might",
    "must",
    "has",
    "have",
    "had",
    "does",
    "do",
    "did",
    "isn't",
    "aren't",
    "wasn't",
    "weren't",
    "can't",
    "couldn't",
    "won't",
    "wouldn't",
    "shouldn't",
    "mayn't",
    "mightn't",
    "mustn't",
    "hasn't",
    "haven't",
    "hadn't",
    "doesn't",
    "don't",
    "didn't",
];

/// Wh-words that can also serve as the subject of a clause.
/// When these are the subject, do-support is NOT needed:
/// "What works best?" "Who came yesterday?"
const WH_SUBJECT_WORDS: &[&str] = &["what", "who", "which"];

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

        // Pattern 3: Missing do-support — wh-word + verb + subject
        // e.g. "What means this?" → "What does this mean?"
        // e.g. "How works it?" → "How does it work?"
        // e.g. "Where starts the program?" → "Where does the program start?"
        // The verb can be either a base-form lemma or 3rd-person singular
        // The subject can be: pronoun, determiner(+noun), or noun
        let missing_do_pronoun = SequenceExpr::word_set(WH_WORDS)
            .t_ws()
            .then_kind_where(|k| {
                (k.is_verb_lemma() || k.is_verb_third_person_singular_present_form())
                    && !k.is_auxiliary_verb()
            })
            .t_ws()
            // Subject is a pronoun: "How works it?"
            .then_pronoun();

        let missing_do_det = SequenceExpr::word_set(WH_WORDS)
            .t_ws()
            .then_kind_where(|k| {
                (k.is_verb_lemma() || k.is_verb_third_person_singular_present_form())
                    && !k.is_auxiliary_verb()
            })
            .t_ws()
            // Subject is a determiner: "Where starts the..."
            .then_determiner()
            .t_ws()
            // Determiner is followed by a noun: "the program"
            .then_nominal();

        let missing_do_noun = SequenceExpr::word_set(WH_WORDS)
            .t_ws()
            .then_kind_where(|k| {
                (k.is_verb_lemma() || k.is_verb_third_person_singular_present_form())
                    && !k.is_auxiliary_verb()
            })
            .t_ws()
            // Subject is a noun: "What means love?"
            .then_noun();

        Self {
            expr: SequenceExpr::any_of(vec![
                Box::new(do_plus_modal),
                Box::new(do_plus_past),
                Box::new(missing_do_pronoun),
                Box::new(missing_do_det),
                Box::new(missing_do_noun),
            ]),
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
                if let Some(last) = without_ed.last()
                    && !last.is_vowel()
                    && without_ed.len() > 1
                {
                    let without_doubled = without_ed[..without_ed.len() - 1].to_vec();
                    if self.is_verb_lemma(&without_doubled) {
                        suggs.push(without_doubled);
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

    /// Check if a word is an auxiliary verb by checking against the known list.
    fn is_aux_word(word: &[char]) -> bool {
        AUXILIARIES.iter().any(|&aux| word.eq_str(aux))
    }

    /// Check if a word is a known wh-word.
    fn is_wh_word(word: &[char]) -> bool {
        WH_WORDS.iter().any(|&w| word.eq_str(w))
    }

    /// Check if a wh-word can also be a subject.
    fn is_wh_subject_word(word: &[char]) -> bool {
        WH_SUBJECT_WORDS.iter().any(|&w| word.eq_str(w))
    }

    /// Get the base form of a verb for use after do-support.
    /// If the word is already a verb_lemma (base form), return it as-is.
    /// If it's a 3rd person singular present form, try to derive the base form.
    fn get_base_verb_form(&self, verb_chars: &[char], _src: &[char]) -> Option<String> {
        let verb_str: String = verb_chars.iter().collect();

        // If it's already a verb lemma, return as-is
        if self.is_verb_lemma(verb_chars) {
            return Some(verb_str);
        }

        // Try to strip 3sg -s/-es/-ies to get base form
        let base = if verb_str.ends_with("ies") && verb_str.len() > 3 {
            // "carries" → "carry", "worries" → "worry"
            let mut base: String = verb_str[..verb_str.len() - 3].to_string();
            base.push('y');
            base
        } else if verb_str.ends_with("sses") && verb_str.len() > 4 {
            // "passes" → "pass", "presses" → "press"
            verb_str[..verb_str.len() - 2].to_string()
        } else if verb_str.ends_with("shes") && verb_str.len() > 4 {
            // "washes" → "wash", "finishes" → "finish"
            verb_str[..verb_str.len() - 2].to_string()
        } else if verb_str.ends_with("ches") && verb_str.len() > 4 {
            // "watches" → "watch", "reaches" → "reach"
            verb_str[..verb_str.len() - 2].to_string()
        } else if verb_str.ends_with("xes") && verb_str.len() > 3 {
            // "fixes" → "fix", "boxes" → "box"
            verb_str[..verb_str.len() - 2].to_string()
        } else if verb_str.ends_with("oes") && verb_str.len() > 3 {
            // "goes" → "go", "does" → "do" — but "does" is handled separately
            let candidate = verb_str[..verb_str.len() - 2].to_string();
            if self.is_verb_lemma(&candidate.chars().collect::<Vec<_>>()) {
                candidate
            } else {
                // Try just stripping -s
                verb_str[..verb_str.len() - 1].to_string()
            }
        } else if verb_str.ends_with('s') && !verb_str.ends_with("ss") && verb_str.len() > 2 {
            // "means" → "mean", "works" → "work", "runs" → "run"
            verb_str[..verb_str.len() - 1].to_string()
        } else {
            verb_str.clone()
        };

        // Verify the base form is a verb lemma
        let base_chars: Vec<char> = base.chars().collect();
        if self.is_verb_lemma(&base_chars) {
            Some(base)
        } else {
            // Return the original as fallback
            Some(verb_str)
        }
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
        // Collect only word tokens from the match (skip whitespace/punctuation)
        let word_toks: Vec<&Token> = toks.iter_words().collect();

        let first_tok = word_toks.first()?;
        let first_word = first_tok.span.get_content(src);

        // ── Pattern 3: Missing do-support ────────────────────────────────
        // wh-word + verb + subject
        // e.g. "What means this" → "What does this mean"
        if Self::is_wh_word(first_word) && word_toks.len() >= 3 {
            let verb_tok = word_toks[1];
            let verb_word = verb_tok.span.get_content(src);
            let _verb_str = verb_tok.span.get_content_string(src);

            // The verb must be a verb (lemma or 3rd-person-singular) and not an auxiliary
            if (verb_tok.kind.is_verb_lemma()
                || verb_tok.kind.is_verb_third_person_singular_present_form())
                && !Self::is_aux_word(verb_word)
            {
                // Determine the subject — it starts at word_toks[2]
                // For determiner subjects, the noun is word_toks[3]
                let subject_start = word_toks[2];
                let is_determiner = subject_start.kind.is_determiner();
                let is_pronoun =
                    subject_start.kind.is_subject_pronoun() || subject_start.kind.is_pronoun();
                let is_noun = subject_start.kind.is_noun();

                // Only proceed if the third word is a valid subject head
                if is_pronoun || is_determiner || is_noun {
                    // Don't flag if NOT sentence-initial (could be indirect question)
                    if let Some((before, _)) = context
                        && !before.is_empty()
                    {
                        return None;
                    }

                    // ── Guard: wh-word as subject ────────────────────
                    // "What works best?" — "what" is the subject here
                    // "What means this?" — "what" is the fronted object, "this" is subject
                    //
                    // Heuristic: if the wh-word can be a subject AND the verb is 3sg
                    // AND the following word is NOT clearly a subject (not pronoun/det),
                    // then the wh-word is likely the subject.
                    if Self::is_wh_subject_word(first_word)
                        && verb_tok.kind.is_verb_third_person_singular_present_form()
                        && !is_pronoun
                        && !is_determiner
                    {
                        // "What works best?" — "what" = subject, "works" = verb, "best" = adverb
                        // Only flag if is_noun is true AND the noun is a proper noun or clearly
                        // not an adverb. For common words that are both noun/adverb (like "best"),
                        // be conservative and skip.
                        if !is_noun {
                            return None;
                        }
                        // Even if it's a noun, check if the wh-word-as-subject reading
                        // is more natural by looking at context: if the "subject" word
                        // is also commonly an adverb, skip.
                        // For simplicity, require pronoun or determiner for wh-subject words.
                        return None;
                    }

                    // Build the subject text
                    // For determiner + noun, include both tokens
                    let (subject_text, last_subject_idx) = if is_determiner
                        && word_toks.len() >= 4
                        && word_toks[3].kind.is_nominal()
                    {
                        // "the program" — combine determiner + noun
                        let det_text: String = subject_start.span.get_content(src).iter().collect();
                        let noun_text: String = word_toks[3].span.get_content(src).iter().collect();
                        (format!("{det_text} {noun_text}"), 3)
                    } else {
                        let text: String = subject_start.span.get_content(src).iter().collect();
                        (text, 2)
                    };

                    // Determine do form based on subject
                    let do_form = if is_pronoun
                        && subject_start.kind.is_third_person_singular_pronoun()
                    {
                        "does"
                    } else if is_determiner {
                        // Determiner + noun phrase — assume singular for "the/a/this"
                        // Could be plural for "these/those/some" but be conservative
                        let det_str: String = subject_start.span.get_content(src).iter().collect();
                        let det_lower = det_str.to_lowercase();
                        if det_lower == "these" || det_lower == "those" || det_lower == "some" {
                            "do"
                        } else {
                            "does"
                        }
                    } else if is_noun && !subject_start.kind.is_plural_noun() {
                        "does"
                    } else {
                        "do"
                    };

                    // Get the text for each component
                    let wh_text: String = first_tok.span.get_content(src).iter().collect();

                    // Build the replacement: "What does this mean?"
                    let base_verb = self.get_base_verb_form(verb_word, src)?;

                    // Find the span to replace: from verb through last subject token
                    let verb_tok_idx = toks.iter().position(|t| t.span == verb_tok.span)?;
                    let last_subject_tok = word_toks[last_subject_idx];
                    let last_subject_tok_idx =
                        toks.iter().position(|t| t.span == last_subject_tok.span)?;

                    // Span covers: "means this" or "starts the program" (verb through last subject)
                    let verb_through_subj_span =
                        toks[verb_tok_idx..=last_subject_tok_idx].span()?;

                    // The replacement text: "does this mean" or "does the program start"
                    let replacement = format!("{do_form} {subject_text} {base_verb}");
                    let replacement_chars: Vec<char> = replacement.chars().collect();

                    return Some(Lint {
                        span: verb_through_subj_span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::replace_with_match_case(
                            replacement_chars,
                            verb_through_subj_span.get_content(src),
                        )],
                        message: format!(
                            "In English, questions with a fronted wh-word need do-support. \
                             Try: \"{wh_text} {do_form} {subject_text} {base_verb}\"."
                        ),
                        priority: 31,
                    });
                }
            }
        }

        // ── Pattern 1 & 2: do + modal / do + past ────────────────────────
        if word_toks.len() < 2 {
            return None;
        }

        let do_tok = word_toks[0];
        let do_word = do_tok.span.get_content_string(src);

        // Check the second word token
        let second_tok = word_toks[1];
        let second_word = second_tok.span.get_content(src);
        let second_str = second_tok.span.get_content_string(src);

        // If second word is a modal or auxiliary, suggest removing "do"
        let is_modal_or_aux = second_tok.kind.is_auxiliary_verb()
            || ModalVerb::without_common_errors()
                .matches(&toks[1..], src)
                .is_some()
            || ModalVerb::without_common_errors()
                .matches(&toks[2..], src)
                .is_some();

        if is_modal_or_aux {
            // "They do can swim" → "They can swim"
            // Remove "do " (the do-word token + following whitespace token)
            let do_and_ws_span = toks[0..2].span()?;
            return Some(Lint {
                span: do_and_ws_span,
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
            if let Some((before, _)) = context
                && (before.is_empty() || (before.len() == 1 && before[0].kind.is_whitespace()))
            {
                return None;
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
        "Detects redundant \"do\" before modals, corrects past-tense verbs to base form after \"do\", \
         and flags missing do-support in questions like \"What means this?\" → \"What does this mean?\""
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
        assert_suggestion_result(
            "He does fried the fish.",
            DoSupport::new(FstDictionary::curated()),
            "He does fry the fish.",
        );
    }

    #[test]
    fn fix_does_logged() {
        assert_suggestion_result(
            "It does logged the error.",
            DoSupport::new(FstDictionary::curated()),
            "It does log the error.",
        );
    }

    // ── Missing do-support in wh-questions ───────────────────────────────

    #[test]
    fn fix_what_means_this() {
        assert_suggestion_result(
            "What means this?",
            DoSupport::new(FstDictionary::curated()),
            "What does this mean?",
        );
    }

    #[test]
    fn fix_how_works_it() {
        assert_suggestion_result(
            "How works it?",
            DoSupport::new(FstDictionary::curated()),
            "How does it work?",
        );
    }

    #[test]
    fn fix_why_fails_the() {
        assert_suggestion_result(
            "Why fails the test?",
            DoSupport::new(FstDictionary::curated()),
            "Why does the test fail?",
        );
    }

    #[test]
    fn fix_where_starts_the() {
        assert_suggestion_result(
            "Where starts the program?",
            DoSupport::new(FstDictionary::curated()),
            "Where does the program start?",
        );
    }

    #[test]
    fn fix_what_cost_this() {
        assert_suggestion_result(
            "What cost this?",
            DoSupport::new(FstDictionary::curated()),
            "What does this cost?",
        );
    }

    #[test]
    fn fix_how_runs_python() {
        assert_suggestion_result(
            "How runs Python?",
            DoSupport::new(FstDictionary::curated()),
            "How does Python run?",
        );
    }

    // ── No false positives ──────────────────────────────────────────────

    #[test]
    fn no_flag_question_did_you() {
        assert_no_lints(
            "Did you went to the store?",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_do_as_main_verb() {
        assert_no_lints(
            "I'll do my homework.",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_emphatic_do() {
        assert_no_lints("I do like pizza.", DoSupport::new(FstDictionary::curated()));
    }

    #[test]
    fn no_flag_do_have_base() {
        assert_no_lints(
            "They do have a point.",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_question_does_it() {
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

    #[test]
    fn no_flag_wh_word_with_auxiliary() {
        assert_no_lints("What is this?", DoSupport::new(FstDictionary::curated()));
    }

    #[test]
    fn no_flag_wh_word_with_modal() {
        assert_no_lints("How can we help?", DoSupport::new(FstDictionary::curated()));
    }

    #[test]
    fn no_flag_means_as_noun() {
        assert_no_lints(
            "What means did they use?",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_indirect_question() {
        assert_no_lints(
            "I wonder what means the most to him.",
            DoSupport::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn no_flag_wh_word_as_subject() {
        assert_no_lints(
            "What works best here?",
            DoSupport::new(FstDictionary::curated()),
        );
    }
}

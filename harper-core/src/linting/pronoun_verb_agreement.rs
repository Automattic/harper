use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

static IRREGULAR: &[(&str, &str)] = &[("don't", "doesn't"), ("have", "has"), ("haven't", "hasn't")];
static NON_MODAL_AUX: &[&str] = &[
    "do", "don't", "does", "doesn't", "have", "has", "haven't", "hasn't", "dont", "doesnt",
    "havent", "hasnt",
];

pub struct PronounVerbAgreement<D> {
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D> PronounVerbAgreement<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        // TODO: allowing "you" leads to false positives:
        // "8 years to give you rewards", "all I can do is give you examples"
        let non_3p_sing_pres_pronoun_with_3p_sing_pres_verb = SequenceExpr::default()
            .then_kind_both_but_not(
                (
                    TokenKind::is_personal_pronoun,
                    TokenKind::is_subject_pronoun,
                ),
                TokenKind::is_third_person_singular_pronoun,
            )
            .t_ws()
            // NOTE: allowing verbs that are also nouns leads to false positives:
            // "Are they colors or colours?"
            // "8 years to give you rewards"
            // "all I can do is give you examples"
            .then_verb_third_person_singular_present_form();

        // NOTE: But excluding them causes many more false positives:
        // boxes, does, drops, flies, gets, goes, likes, site, wakes
        // .then_kind_where(|k| k.is_verb_third_person_singular_present_form() && !k.is_plural_noun());

        let third_person_sing_pres_pronoun = |t: &Token, _: &[char]| {
            t.kind.is_subject_pronoun()
                && !t.kind.is_object_pronoun()
                && t.kind.is_personal_pronoun()
                && t.kind.is_third_person_singular_pronoun()
                && !t.kind.is_plural_pronoun()
        };

        let verb_lemma = |t: &Token, src: &[char]| {
            t.kind.is_verb_lemma()
                && !t.kind.is_verb_third_person_singular_present_form()
                && !t.kind.is_verb_simple_past_form() // eg. not "put"
                && !t.kind.is_adverb() // eg. not "even"
                && !t.kind.is_conjunction() // "and"
                && (!t.kind.is_auxiliary_verb() // "I go"≠"he goes" but "I can"="he can"
                // We don't want modals because they don't inflect, but we want the other auxiliaries.
                || t.span.get_content(src).eq_any_ignore_ascii_case_str(NON_MODAL_AUX))
        };

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                // One Expr for the "I walks" type:
                Box::new(non_3p_sing_pres_pronoun_with_3p_sing_pres_verb),
                // Two Expr's for the "he walk" type:
                Box::new(
                    SequenceExpr::with(third_person_sing_pres_pronoun)
                        .t_ws()
                        .then(verb_lemma),
                ),
                Box::new(SequenceExpr::aco("it").t_ws().t_aco("don't")),
            ])),
            dict,
        }
    }

    fn third_person_singular_present_to_lemma(&self, form: &[char]) -> Vec<Vec<char>> {
        let mut words: Vec<Vec<char>> = Vec::new();

        // -s
        if form.ends_with_ignore_ascii_case_chars(&['s']) {
            words.push(form[0..form.len() - 1].to_vec());

            // -es
            if form.ends_with_ignore_ascii_case_chars(&['e', 's']) {
                words.push(form[0..form.len() - 2].to_vec());

                // -ies -> -y
                if form.ends_with_ignore_ascii_case_chars(&['i', 'e', 's']) {
                    words.push(
                        format!("{}y", &form[0..form.len() - 3].iter().collect::<String>())
                            .chars()
                            .collect(),
                    );
                }
            }
        }

        if let Some((lemma, _)) = IRREGULAR
            .iter()
            .find(|(_, f)| form.eq_ignore_ascii_case_str(f))
        {
            words.push(lemma.chars().collect::<Vec<char>>());
        }

        words
            .iter()
            .filter(|&w| {
                self.dict
                    .get_word_metadata(w)
                    .is_some_and(|md| md.is_verb_lemma())
            })
            .map(|w| w.to_vec())
            .collect()
    }

    fn lemma_to_third_person_singular_present(&self, input: &str) -> Vec<Vec<char>> {
        let mut words: Vec<Vec<char>> = Vec::new();

        words.push(format!("{input}s").chars().collect());
        words.push(format!("{input}es").chars().collect());

        if input.ends_with("y") {
            words.push(
                format!("{}ies", &input[0..input.len() - 1])
                    .chars()
                    .collect(),
            );
        }

        if let Some((_, form)) = IRREGULAR
            .iter()
            .find(|(lemma, _)| input.eq_ignore_ascii_case(lemma))
        {
            words.push(form.chars().collect());
        }

        words
            .iter()
            .filter(|&w| {
                self.dict
                    .get_word_metadata(w)
                    .is_some_and(|md| md.is_verb_third_person_singular_present_form())
            })
            .map(|w| w.to_vec())
            .collect()
    }
}

impl<D> ExprLinter for PronounVerbAgreement<D>
where
    D: Dictionary,
{
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let pron_tok = &toks[0];
        let is_3psg = pron_tok.kind.is_third_person_singular_pronoun();

        if is_3psg
            && let Some((before, _)) = ctx
            && let [.., prev_word_tok, ws_tok] = before
        {
            let is_exempt = ws_tok.kind.is_whitespace()
                // Don't flag "can he go", "does he go", etc.
                && (prev_word_tok.kind.is_auxiliary_verb()
                    || prev_word_tok
                        .span
                        .get_content(src)
                        .eq_any_ignore_ascii_case_str(&[
                            // Note: "if he go" etc. may sound archaic or wrong to us but is in the US constitution
                            // "if",
                            "that",
                            // Some words make a following "that" optional:
                            "insisted",
                            "suggested",
                        ]));
            if is_exempt {
                return None;
            }
        }

        let verb_tok = toks.last()?;
        let verb_span = verb_tok.span;
        let verb_chars = verb_tok.span.get_content(src);
        let verb_str = verb_tok.span.get_content_string(src);

        let suggs = if is_3psg {
            self.lemma_to_third_person_singular_present(&verb_str)
        } else {
            self.third_person_singular_present_to_lemma(verb_chars)
        };

        let suggestions = suggs
            .into_iter()
            .map(|s| Suggestion::replace_with_match_case(s, verb_chars))
            .collect();

        Some(Lint {
            span: verb_span,
            lint_kind: LintKind::Agreement,
            suggestions,
            message: "The form of the verb must agree in grammatical number with the pronoun."
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Ensures pronouns agree with their verbs."
    }
}

#[cfg(test)]
mod lints {
    use super::PronounVerbAgreement;
    use crate::linting::tests::{
        assert_no_lints, assert_suggestion_result, assert_top3_suggestion_result,
    };
    use crate::spell::FstDictionary;

    // Expected to be fixed, but there are exceptions

    #[test]
    fn issue_233_1() {
        assert_suggestion_result(
            "I likes this place.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "I like this place.",
        );
    }

    #[test]
    fn issue_233_2() {
        assert_suggestion_result(
            "I sits under the AC.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "I sit under the AC.",
        );
    }

    #[test]
    #[ignore = "because 'like' is an adjective as well as a verb."]
    fn issue_233_1_reverse() {
        assert_suggestion_result(
            "He like this place.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "He likes this place.",
        );
    }

    #[test]
    fn why_we_cant_flag_like_yet() {
        assert_no_lints(
            "What is he like?",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn issue_233_2_reverse() {
        assert_top3_suggestion_result(
            "She sit under the AC.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "She sits under the AC.",
        );
    }

    #[test]
    fn dont_flag_correct_agreement() {
        assert_no_lints(
            "He likes this place. I sit under the AC.",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    // Every pronoun systematically

    // Expected to get corrected

    #[test]
    fn fixes_i() {
        assert_suggestion_result(
            "I wakes up.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "I wake up.",
        );
    }

    #[test]
    fn fixes_we() {
        assert_suggestion_result(
            "We gets dressed.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "We get dressed.",
        );
    }

    #[test]
    fn fixes_you() {
        assert_suggestion_result(
            "You drops off the kids.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "You drop off the kids.",
        );
    }

    #[test]
    fn fixes_he() {
        assert_suggestion_result(
            "He work hard.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "He works hard.",
        );
    }

    #[test]
    fn fixes_she() {
        assert_suggestion_result(
            "She study hard.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "She studies hard.",
        );
    }

    #[test]
    #[ignore = "Becasue 'it' is also object case. Eg. 'watch it break down'"]
    fn we_cant_fix_it_yet() {
        assert_suggestion_result(
            "It break down.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "It breaks down.",
        );
    }

    #[test]
    fn why_we_cant_fix_it_yet() {
        assert_no_lints(
            "I heard it break down.",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn fixes_they() {
        assert_suggestion_result(
            "They repairs it.",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "They repair it.",
        )
    }

    // Correct phrases that are expected not to get corrected

    #[test]
    fn dont_flag_i() {
        assert_no_lints("I eat", PronounVerbAgreement::new(FstDictionary::curated()));
    }

    #[test]
    fn dont_flag_we() {
        assert_no_lints(
            "We drink",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_you() {
        assert_no_lints(
            "You walk",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_he() {
        assert_no_lints(
            "He runs",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_she() {
        assert_no_lints(
            "She swims",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_it() {
        assert_no_lints(
            "It works!",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_they() {
        assert_no_lints(
            "They finish",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    // Ceck changing verb endings

    // -ies ↔ -y
    #[test]
    fn fix_flies() {
        assert_suggestion_result(
            "I flies",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "I fly",
        );
    }
    #[test]
    fn fix_cry() {
        assert_suggestion_result(
            "He cry",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "He cries",
        );
    }

    // -o ↔ -oes
    #[test]
    fn fix_go() {
        assert_suggestion_result(
            "She go",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "She goes",
        );
    }
    #[test]
    fn fix_goes() {
        assert_suggestion_result(
            "They goes",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "They go",
        );
    }

    // Check irregular changes

    // has ↔ have
    #[test]
    fn fix_has() {
        assert_suggestion_result(
            "You has",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "You have",
        );
    }
    #[test]
    fn fix_have() {
        assert_suggestion_result(
            "She have",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "She has",
        );
    }

    // hasn't ↔ haven't
    #[test]
    fn fix_hasnt() {
        assert_suggestion_result(
            "You hasn't",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "You haven't",
        );
    }
    #[test]
    fn fix_havent() {
        assert_suggestion_result(
            "He haven't",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "He hasn't",
        );
    }

    // -es
    #[test]
    fn fix_box() {
        assert_suggestion_result(
            "He box",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "He boxes",
        );
    }
    #[test]
    fn fix_boxes() {
        assert_suggestion_result(
            "You boxes",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "You box",
        );
    }

    // TODO: Are there any double consonant endings to change?
    // TODO: Are there any f ↔ v endings to change?

    // Negative contractions

    // doesn't ↔ don't
    #[test]
    fn fix_doesnt() {
        assert_suggestion_result(
            "We doesn't",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "We don't",
        );
    }
    #[test]
    // Note: This requires a dedicated branch of the `[Expr]`
    fn fix_dont() {
        assert_suggestion_result(
            "It don't",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "It doesn't",
        );
    }

    // Does do ↔ does behave differently to box ↔ boxes due to being an auxiliary verb?
    #[test]
    fn fix_do() {
        assert_suggestion_result(
            "He do",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "He does",
        );
    }
    #[test]
    fn fix_does() {
        assert_suggestion_result(
            "You does",
            PronounVerbAgreement::new(FstDictionary::curated()),
            "You do",
        );
    }

    // False positives found by Elijah

    #[test]
    fn false_positive_she_consider() {
        assert_no_lints(
            "On April 10th, I suggested she consider a smaller, more intimate gathering.",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn false_positive_she_sell() {
        assert_no_lints(
            "I suggested she sell it and use the proceeds to help with her relocation expenses, or perhaps rent a similar camera while in Barcelona.",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn false_positive_she_rent() {
        assert_no_lints(
            "I suggested she sell it and use the proceeds to help with her relocation expenses, or perhaps rent a similar camera while in Barcelona.",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn false_positive_he_donned() {
        assert_no_lints(
            "He donned his heavy oilskins and descended the winding staircase, his boots echoing in the hollow tower.",
            PronounVerbAgreement::new(FstDictionary::curated()),
        );
    }
}

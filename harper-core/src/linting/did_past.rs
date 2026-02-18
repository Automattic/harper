use crate::{
    CharStringExt, Lint, Token,
    char_ext::CharExt,
    expr::{Expr, FixedPhrase, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
    spell::Dictionary,
};

pub struct DidPast<D> {
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D> DidPast<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::longest_of(vec![
                    Box::new(WordSet::new(&["did", "didn't", "didnt"])),
                    Box::new(FixedPhrase::from_phrase("did not")),
                ])
                .then_optional(SequenceExpr::default().t_ws().then_subject_pronoun())
                .t_ws()
                // Note that 'simple past forms' may apply only to irregular verbs
                // Note and that 'past forms' applies to regular verbs where preterite and participle share a form
                .then_kind_where(|k| {
                    (k.is_verb_simple_past_form() || k.is_verb_past_form()) && !k.is_verb_lemma()
                }),
            ),
            dict,
        }
    }

    fn keep_suggestion_if_lemma(&self, suggs: &mut Vec<Vec<char>>, candidate: &[char]) {
        if self
            .dict
            .get_word_metadata(candidate)
            .is_some_and(|md| md.is_verb_lemma())
        {
            suggs.push(candidate.to_vec());
        }
    }
}

impl<D> ExprLinter for DidPast<D>
where
    D: Dictionary,
{
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects past forms of verbs to their base form, when used together with \"did\"."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let vspan = toks.last()?.span;
        let vchars = vspan.get_content(src);
        let vstr = vspan.get_content_string(src);

        let mut suggs = vec![];

        // Chop -d/-ed off regular verbs

        if vchars.ends_with_ignore_ascii_case_chars(&['d']) {
            let without_d = &vchars[..vchars.len() - 1];

            if without_d.ends_with_ignore_ascii_case_chars(&['e']) {
                let without_ed = &without_d[..without_d.len() - 1];

                self.keep_suggestion_if_lemma(&mut suggs, without_ed);

                // If the stem without -ed now ends in -i, try changing that to -y to find the lemma
                if without_ed.ends_with_ignore_ascii_case_chars(&['i']) {
                    let mut with_final_y = without_ed[..without_ed.len() - 1].to_vec();
                    with_final_y.push('y');
                    self.keep_suggestion_if_lemma(&mut suggs, &with_final_y);
                }

                // If the stem without -ed ends in a doubled consonant, try with just a single one
                if without_ed.last().is_some_and(|c| !c.is_vowel()) {
                    let without_doubled_consonant = without_ed[..without_ed.len() - 1].to_vec();
                    self.keep_suggestion_if_lemma(&mut suggs, &without_doubled_consonant);
                }
            }
            self.keep_suggestion_if_lemma(&mut suggs, without_d);
        }

        // Look up irregular verbs

        if let Some(lemma) = IrregularVerbs::curated().get_lemma_for_preterite(&vstr) {
            suggs.push(lemma.chars().collect());
        }

        if !suggs.is_empty() {
            Some(Lint {
                span: vspan,
                lint_kind: LintKind::Redundancy,
                suggestions: suggs
                    .into_iter()
                    .map(|s| Suggestion::replace_with_match_case(s, vchars))
                    .collect(),
                message: "Use the base form of the verb with \"did\".".to_string(),
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
    use crate::{
        linting::tests::{assert_no_lints, assert_suggestion_result},
        spell::FstDictionary,
    };

    // Test basic 'true positive' regular verb cases

    // Regular verb where past is lemma+ed

    #[test]
    fn ed_did_forked() {
        assert_suggestion_result(
            "Did they forked the repo?",
            DidPast::new(FstDictionary::curated()),
            "Did they fork the repo?",
        );
    }

    // Regular verb where past is lemma+d

    #[test]
    fn d_did_used() {
        assert_suggestion_result(
            "It didn't used a macro.",
            DidPast::new(FstDictionary::curated()),
            "It didn't use a macro.",
        );
    }

    // Regular verb where past is lemma -y +ied

    #[test]
    fn y_did_fried() {
        assert_suggestion_result(
            "I hope that didn't fried any chips!",
            DidPast::new(FstDictionary::curated()),
            "I hope that didn't fry any chips!",
        );
    }

    // Regular verb where past doubled the final consonant

    #[test]
    fn doubed_consonant_logged() {
        assert_suggestion_result(
            "There was a segfault but it did logged the error.",
            DidPast::new(FstDictionary::curated()),
            "There was a segfault but it did log the error.",
        );
    }

    // Test basic 'true positive' irregular verb cases

    #[test]
    fn did_past() {
        assert_suggestion_result("Did went", DidPast::new(FstDictionary::curated()), "Did go");
    }

    #[test]
    fn did_past_with_apostrophe() {
        assert_suggestion_result(
            "Didn't saw",
            DidPast::new(FstDictionary::curated()),
            "Didn't see",
        );
    }

    #[test]
    fn didnt_past_no_apostrophe() {
        assert_suggestion_result(
            "Didnt had",
            DidPast::new(FstDictionary::curated()),
            "Didnt have",
        );
    }

    #[test]
    fn did_i_heard() {
        assert_suggestion_result(
            "Did I heard",
            DidPast::new(FstDictionary::curated()),
            "Did I hear",
        );
    }

    #[test]
    fn did_i_heard_with_apostrophe() {
        assert_suggestion_result(
            "Didn't we heard",
            DidPast::new(FstDictionary::curated()),
            "Didn't we hear",
        );
    }

    #[test]
    fn didnt_i_forgot_no_apostrophe() {
        assert_suggestion_result(
            "Didnt he forgot",
            DidPast::new(FstDictionary::curated()),
            "Didnt he forget",
        );
    }

    // Test basic 'true negative' cases - verb is valid as both lemma and simple past

    #[test]
    fn ignore_lemma_same_as_past_tense() {
        assert_no_lints("Did read", DidPast::new(FstDictionary::curated()));
    }

    // Real-world examples

    #[test]
    fn fix_did_you_cmae() {
        assert_suggestion_result(
            "How did you came to this",
            DidPast::new(FstDictionary::curated()),
            "How did you come to this",
        );
    }

    #[test]
    fn fix_did_you_wrote() {
        assert_suggestion_result(
            "I'm very interested in the script, if you did wrote it.",
            DidPast::new(FstDictionary::curated()),
            "I'm very interested in the script, if you did write it.",
        );
    }

    #[test]
    fn fix_didnt_had() {
        assert_suggestion_result(
            "and i DO know that i didnt had any Terracota",
            DidPast::new(FstDictionary::curated()),
            "and i DO know that i didnt have any Terracota",
        );
    }

    #[test]
    fn did_you_went() {
        assert_suggestion_result(
            "Did you went out of memory maybe?",
            DidPast::new(FstDictionary::curated()),
            "Did you go out of memory maybe?",
        );
    }

    #[test]
    fn fix_did_needed() {
        assert_suggestion_result(
            "since our CI was broken this did needed to be done",
            DidPast::new(FstDictionary::curated()),
            "since our CI was broken this did need to be done",
        );
    }

    #[test]
    fn fix_did_thought() {
        assert_suggestion_result(
            "I did thought of adding it as a tooltip on hover",
            DidPast::new(FstDictionary::curated()),
            "I did think of adding it as a tooltip on hover",
        );
    }

    #[test]
    fn fix_did_wanted() {
        assert_suggestion_result(
            "I did wanted catch all errors in my previous example.",
            DidPast::new(FstDictionary::curated()),
            "I did want catch all errors in my previous example.",
        );
    }

    #[test]
    fn fix_did_not_changed() {
        assert_suggestion_result(
            "freeing space and reboot frequently did not changed anything",
            DidPast::new(FstDictionary::curated()),
            "freeing space and reboot frequently did not change anything",
        );
    }

    #[test]
    fn ignore_did_you_read() {
        assert_no_lints(
            "Did You Read the Instructions?",
            DidPast::new(FstDictionary::curated()),
        );
    }
}

use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
    spell::Dictionary,
};

pub struct WillNonLemma<D>
where
    D: Dictionary,
{
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D: Dictionary> WillNonLemma<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["will", "shall"])
                    .t_ws()
                    .then_kind_where(|kind| {
                        kind.is_verb()
                            // flag "will walked/walks/walking" but not "will walk" (lemma)
                            && !kind.is_verb_lemma()
                            // avoid flagging "will drinks be expensive" ("drinks" is a noun in context)
                            // but "coming" is also a noun since it's a gerund and we do want to flag "will coming next soon"
                            && (!kind.is_noun() || kind.is_verb_progressive_form())
                            // avoid flagging "will was read" ("will" is a noun in context this time)
                            && !(kind.is_linking_verb() && kind.is_verb_simple_past_form())
                    }),
            ),
            dict,
        }
    }
}

impl<D: Dictionary> ExprLinter for WillNonLemma<D> {
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
        let matched_chars = toks.span()?.get_content(src);

        // 'modal' is the 3rd last token, verb is the last token
        let (fut_idx, verb_idx) = (0, toks.len() - 1);
        let (fut_tok, verb_tok) = (&toks[fut_idx], &toks[verb_idx]);
        let verb_str = verb_tok.span.get_content_string(src);

        // Disambiguate 'will' noun from 'will' verb: Preceding "a", "the", "my", "your", etc. show it's a noun.
        if fut_tok.get_ch(src).eq_ch(&['w', 'i', 'l', 'l'])
            && preceded_by_word(ctx, |tok| tok.kind.is_determiner())
        {
            return None;
        }

        let suggest =
            |text: &str| Suggestion::replace_with_match_case(text.chars().collect(), matched_chars);

        let maybe_prev_word_tok: Option<&Token> = match ctx {
            Some((prev, _)) if prev.len() >= 2 => {
                let last = &prev[prev.len() - 1];
                let potential_word = &prev[prev.len() - 2];
                if last.kind.is_whitespace() && potential_word.kind.is_word() {
                    Some(potential_word)
                } else {
                    None
                }
            }
            _ => None,
        };

        let mut suggestions = vec![];

        if verb_tok.kind.is_verb_simple_past_form()
            && let Some(lemma) = IrregularVerbs::curated().get_lemma_for_preterite(&verb_str)
            && self
                .dict
                .get_word_metadata_str(lemma)
                .is_some_and(|m| m.is_verb_lemma())
        {
            suggestions.push(suggest(&format!("will {}", lemma)));
            suggestions.push(suggest(&verb_str));
        }
        if verb_tok.kind.is_verb_third_person_singular_present_form() {
            let candidate = &verb_str[..verb_str.len() - 1];
            if self
                .dict
                .get_word_metadata_str(candidate)
                .is_some_and(|m| m.is_verb_lemma())
            {
                suggestions.push(suggest(&format!("will {}", candidate)));
                suggestions.push(suggest(&verb_str));

                // Add suggestion for plural nouns
                if maybe_prev_word_tok.is_some_and(|tok| tok.kind.is_plural_nominal()) {
                    suggestions.push(suggest(candidate));
                }
            }
        }
        if verb_tok.kind.is_verb_progressive_form() {
            if let Some(stem) = verb_str.strip_suffix("ing") {
                // Check regular form (e.g., 'walking' -> 'walk')
                if self
                    .dict
                    .get_word_metadata_str(stem)
                    .is_some_and(|m| m.is_verb_lemma())
                {
                    suggestions.push(Suggestion::replace_with_match_case(
                        format!("will {}", stem).chars().collect(),
                        matched_chars,
                    ));
                }

                // Check form that adds 'e' (e.g., 'coming' -> 'come')
                let stem_with_e = format!("{}e", stem);
                if self
                    .dict
                    .get_word_metadata_str(&stem_with_e)
                    .is_some_and(|m| m.is_verb_lemma())
                {
                    suggestions.push(Suggestion::replace_with_match_case(
                        format!("will {}", stem_with_e).chars().collect(),
                        matched_chars,
                    ));
                }
            }

            let v_ing = Suggestion::replace_with_match_case(
                verb_tok.span.get_content(src).to_vec(),
                toks.span()?.get_content(src),
            );
            suggestions.push(v_ing);
            let will_be_v_ing = Suggestion::replace_with_match_case(
                format!("will be {}", verb_str)
                    .chars()
                    .collect::<Vec<char>>(),
                toks.span()?.get_content(src),
            );
            suggestions.push(will_be_v_ing);
        }

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Grammar,
            suggestions,
            message: "`Will` and `shall` should be followed by a verb in its base form."
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Flags wrong verb forms after `will` or `shall`"
    }
}

#[cfg(test)]
mod tests {
    use super::WillNonLemma;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_lint_count};
    use crate::spell::FstDictionary;

    #[test]
    fn fix_will_ran() {
        // singular + will + irregular preterite
        assert_good_and_bad_suggestions(
            "The brown fox will ran thru the meadow.",
            WillNonLemma::new(FstDictionary::curated()),
            &[
                "The brown fox will run thru the meadow.",
                "The brown fox ran thru the meadow.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_will_exists() {
        // plural + will + 3rd person singular present
        assert_good_and_bad_suggestions(
            "there is a good chance duplicate Rule IDs will exists.",
            WillNonLemma::new(FstDictionary::curated()),
            &[
                "there is a good chance duplicate Rule IDs will exist.",
                "there is a good chance duplicate Rule IDs exists.",
                "there is a good chance duplicate Rule IDs exist.",
            ],
            &[],
        );
    }

    #[test]
    fn ignore_shall_vessels() {
        // "nor" + shall + (3rd person singular present == plural noun)
        assert_lint_count(
            "No Preference shall be given by any Regulation of Commerce or Revenue to the Ports of one State over those of another; nor shall Vessels bound to, or from, one State, be obliged to enter, clear, or pay Duties in another.",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn ignore_will_tools() {
        // "free will" + (3rd person singular present == plural noun)
        assert_lint_count(
            "Give your AI free will tools.",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn fix_will_coming_soon() {
        // plural + will + progressive
        assert_good_and_bad_suggestions(
            "More advanced features will coming soon, so stay tuned!",
            WillNonLemma::new(FstDictionary::curated()),
            &[
                "More advanced features will come soon, so stay tuned!",
                "More advanced features coming soon, so stay tuned!",
                "More advanced features will be coming soon, so stay tuned!",
            ],
            &[],
        );
    }

    #[test]
    fn fix_will_coming_next() {
        // singular + will + progressive
        assert_good_and_bad_suggestions(
            "on CPU and GPU (NPU support will coming next)",
            WillNonLemma::new(FstDictionary::curated()),
            &[
                "on CPU and GPU (NPU support will come next)",
                "on CPU and GPU (NPU support coming next)",
                "on CPU and GPU (NPU support will be coming next)",
            ],
            &[],
        );
    }

    #[test]
    fn ignore_will_was_read_pr_review() {
        assert_lint_count(
            "Around November 2023, shortly after the will was read, Eleanor started asking about using the cottage.",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn ignore_will_was_straightforward_pr_review() {
        assert_lint_count(
            "Vivian’s will was straightforward. The house, all its contents, and her savings were to be left to me.",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn ignore_will_specifies_equal_responsibility() {
        assert_lint_count(
            "I also reminded her that the will specifies equal responsibility",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn ignore_will_reflected_that() {
        assert_lint_count(
            "She meticulously planned everything, and her will reflected that.",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn ignore_will_stipulates_everything() {
        assert_lint_count(
            "Now, his will stipulates everything is split 50/50.",
            WillNonLemma::new(FstDictionary::curated()),
            0,
        );
    }
}

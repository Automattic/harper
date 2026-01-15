use hashbrown::HashMap;

use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::spell::Dictionary;
use crate::{Lint, Token, TokenStringExt};

/// Maps irregular simple past verb forms to their lemma forms
const IRREGULAR_VERBS: &[(&str, &str)] = &[
    ("ate", "eat"),
    ("awoke", "awake"),
    ("broke", "break"),
    ("burnt", "burn"),
    ("came", "come"),
    ("did", "do"),
    ("dove", "dive"),
    ("drank", "drink"),
    ("drove", "drive"),
    ("flew", "fly"),
    ("forwent", "forgo"),
    ("froze", "freeze"),
    ("got", "get"),
    ("had", "have"),
    ("hit", "hit"),
    // ("hurt", "hurt"),
    ("knew", "know"),
    ("laid", "lay"),
    ("lit", "light"),
    ("lost", "lose"),
    ("made", "make"),
    ("mistook", "mistake"),
    ("overthrew", "overthrow"),
    ("overtook", "overtake"),
    ("overwrote", "overwrite"),
    ("ran", "run"),
    // ("read", "read"),
    ("redid", "redo"),
    // ("reread", "reread"),
    ("rode", "ride"),
    ("rose", "rise"),
    ("saw", "see"),
    ("taught", "teach"),
    ("thought", "think"),
    ("threw", "throw"),
    ("took", "take"),
    ("tore", "tear"),
    ("undid", "undo"),
    ("went", "go"),
    ("wore", "wear"),
    ("wrote", "write"),
];

lazy_static::lazy_static! {
    static ref IRREGULAR_VERB_MAP: HashMap<&'static str, &'static str> =
        IRREGULAR_VERBS.iter().copied().collect();
}

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
                            && !kind.is_verb_lemma()
                            && (!kind.is_noun() || kind.is_verb_progressive_form())
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
        let verb_idx = toks.len() - 1;
        let verb_tok = &toks[verb_idx];
        let verb_str = verb_tok.span.get_content_string(src);

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
            && let Some(&lemma) = IRREGULAR_VERB_MAP.get(verb_str.as_str())
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
}

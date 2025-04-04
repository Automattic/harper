use crate::{Dialect, Dictionary, Document, Span, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct InflectedVerbAfterTo<T>
where
    T: Dictionary,
{
    dictionary: T,
    dialect: Dialect,
}

impl<T: Dictionary> InflectedVerbAfterTo<T> {
    pub fn new(dictionary: T, dialect: Dialect) -> Self {
        Self {
            dictionary,
            dialect,
        }
    }
}

impl<T: Dictionary> Linter for InflectedVerbAfterTo<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        for pi in document.iter_preposition_indices() {
            let prep = document.get_token(pi).unwrap();
            let Some(space) = document.get_token(pi + 1) else {
                continue;
            };
            let Some(word) = document.get_token(pi + 2) else {
                continue;
            };
            if !space.kind.is_whitespace() || !word.kind.is_word() {
                continue;
            }
            let prep_to = document.get_span_content(&prep.span);
            if prep_to != ['t', 'o'] && prep_to != ['T', 'o'] {
                continue;
            }

            let chars = document.get_span_content(&word.span);
            let (len, form) = match word.kind {
                _ if word.kind.is_verb() => match chars {
                    // breaks the Laravel test at "prior to deploying the application"
                    // [.., 'i', 'n', 'g'] => (3, "continuous"),
                    // TODO: needs to handle both -d and -ed (smile-d, frown-ed)
                    [.., 'e', 'd'] => (2, "past"),
                    // TODO: needs to handle both -s and -es (throw-s, catch-es)
                    [.., 's'] => (1, "3rd person singular present"),
                    _ => continue,
                },
                // 3ps pres. verbs currently get wrong metadata from the affix engine!
                _ if word.kind.is_plural_noun() => match chars {
                    // TODO: as above, needs to handle both -s and -es
                    [.., 's'] => (1, "3rd person singular present"), // can use "plural" here for debugging
                    _ => continue,
                },
                _ => continue,
            };
            let stem = chars[..chars.len() - len].to_vec();
            // let dbg_word: String = chars.iter().collect::<String>();
            // let dbg_stem: String = stem.iter().collect();
            let Some(md) = self.dictionary.get_word_metadata(&stem) else {
                // eprintln!(">>>> '{}': Stem '{}' not found", dbg_word, dbg_stem);
                continue;
            };
            if !md.is_verb() {
                // eprintln!(">>>> '{}': Stem '{}' is not a verb", dbg_word, dbg_stem);
                continue;
            }
            if word.kind.is_plural_noun() && md.is_noun() {
                // eprintln!(
                //     ">>>> '{}' is a plural noun. But '{}' is a noun",
                //     dbg_word, dbg_stem
                // );
                continue;
            }

            lints.push(Lint {
                span: Span::new(prep.span.start, word.span.end),
                lint_kind: LintKind::WordChoice,
                message: format!("This verb seems to be in the {} form.", form).to_string(),
                suggestions: vec![Suggestion::ReplaceWith(
                    prep_to
                        .iter()
                        .chain([' '].iter())
                        .chain(stem.iter())
                        .copied()
                        .collect(),
                )],
                ..Default::default()
            });
        }
        lints
    }

    fn description(&self) -> &str {
        "This rule looks for `to verb` where `verb` is not in the infinitive form."
    }
}

#[cfg(test)]
mod tests {
    use super::InflectedVerbAfterTo;
    use crate::{
        Dialect, FstDictionary,
        linting::tests::{assert_lint_count, assert_suggestion_result},
    };

    #[test]
    fn dont_flag_to_check_both_verb_and_noun() {
        assert_lint_count(
            "to check",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn dont_flag_to_checks_both_verb_and_noun() {
        assert_lint_count(
            "to checks",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn dont_flag_to_cheques_not_a_verb() {
        assert_lint_count(
            "to cheques",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    // -ing forms can act as nouns, current heuristics cannot distinguish
    // #[test]
    // fn flag_to_checking() {
    //     assert_lint_count("to checking", InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American), 1);
    // }

    #[test]
    fn flag_check_ed() {
        assert_lint_count(
            "to checked",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            1,
        );
    }

    #[test]
    fn dont_flag_noun_belief_s() {
        assert_lint_count(
            "to beliefs",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn dont_flag_noun_meat_s() {
        assert_lint_count(
            "to meats",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    // #[test]
    // fn check_993_suggestions() {
    //     assert_suggestion_result(
    //         "A location-agnostic structure that attempts to captures the context and content that a Lint occurred.",
    //         InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
    //         "A location-agnostic structure that attempts to capture the context and content that a Lint occurred.",
    //     );
    // }

    #[test]
    fn dont_flag_embarrass_not_in_dictionary() {
        assert_lint_count(
            "Second I'm going to embarrass you for a.",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn corrects_exist_s() {
        assert_suggestion_result(
            "A valid solution is expected to exists.",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            "A valid solution is expected to exist.",
        );
    }

    // TODO: possible once we can check both catche_s and catch_es
    // #[test]
    // fn corrects_es_ending() {
    //     assert_suggestion_result(
    //         "I need it to catches every exception.",
    //         InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
    //         "I need it to catch every exception.",
    //     );
    // }

    // TODO: dict has expand with D flag but expanded is not granted verb status
    // #[test]
    // fn corrects_ed_ending() {
    //     assert_suggestion_result(
    //         "I had to expanded my horizon.",
    //         InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
    //         "I had to expand my horizon.",
    //     );
    // }

    // TODO: possible once we can check both expir_ed and expire_d
    // #[test]
    // fn flags_expire_d() {
    //     assert_lint_count(
    //         "I didn't know it was going to expired.",
    //         InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
    //         1,
    //     );
    // }

    #[test]
    fn corrects_explain_ed() {
        assert_suggestion_result(
            "To explained the rules to the team.",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            "To explain the rules to the team.",
        );
    }

    // TODO: possible once we can check both explor_ed and explore_d
    // #[test]
    // fn corrects_explor_ed() {
    //     assert_suggestion_result(
    //         "I went to explored distant galaxies.",
    //         InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
    //         "I went to explore distant galaxies.",
    //     );
    // }

    #[test]
    fn cant_flag_express_ed_also_noun() {
        assert_lint_count(
            "I failed to clearly expressed my point.",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn correct_feign_ed() {
        assert_suggestion_result(
            "I was able to feigned ignorance.",
            InflectedVerbAfterTo::new(FstDictionary::curated(), Dialect::American),
            "I was able to feign ignorance.",
        );
    }
}

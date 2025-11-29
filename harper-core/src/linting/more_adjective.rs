use itertools::Itertools;

use crate::expr::{Expr, SequenceExpr};
use crate::linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk};
use crate::spell::Dictionary;
use crate::{CharStringExt, Lint, Token, TokenStringExt};

const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];

pub struct MoreAdjective<D> {
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D> MoreAdjective<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["more", "most"])
                    .t_ws()
                    .then_positive_adjective(),
            ),
            dict,
        }
    }

    fn add_valid_candidate(&self, candidates: &mut Vec<String>, candidate: String) -> bool {
        if let Some(metadata) = self.dict.get_word_metadata_str(&candidate)
            && (metadata.is_comparative_adjective() || metadata.is_superlative_adjective())
        {
            candidates.push(candidate);
            true
        } else {
            false
        }
    }
}

impl<D> ExprLinter for MoreAdjective<D>
where
    D: Dictionary,
{
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // Check invariants just in case the Expr changes
        if toks.len() != 3 || !toks[1].kind.is_whitespace() || !toks[2].kind.is_positive_adjective()
        {
            return None;
        }

        let phrase = toks.span()?;

        enum Degree {
            Comparative,
            Superlative,
        }

        let degree_tok = &toks[0];
        let degree_chars = degree_tok.span.get_content(src);

        let degree = if degree_chars.eq_ignore_ascii_case_str("more") {
            Degree::Comparative
        } else if degree_chars.eq_ignore_ascii_case_str("most") {
            Degree::Superlative
        } else {
            return None;
        };

        let ending = match degree {
            Degree::Comparative => "er",
            Degree::Superlative => "est",
        };

        let adj_tok = &toks[2];
        let adj_span = adj_tok.span;
        let adj_chars = adj_span.get_content(src);
        let adj_str = adj_span.get_content_string(src);

        if adj_chars.len() < 2 {
            return None;
        }

        // "humaner" = "more humane", not "more human"
        if adj_str == "human" {
            return None;
        }

        let mut candidates: Vec<String> = vec![];

        // Only a handful of adjectives are irregular
        let new_candidates = match adj_str.as_str() {
            "bad" => match degree {
                Degree::Comparative => Some(&["worse"][..]),
                Degree::Superlative => Some(&["worst"][..]),
            },
            "good" => match degree {
                Degree::Comparative => Some(&["better"][..]),
                Degree::Superlative => Some(&["best"][..]),
            },
            "far" => match degree {
                Degree::Comparative => Some(&["further", "farther"][..]),
                Degree::Superlative => Some(&["furthest", "farthest"][..]),
            },
            _ => None,
        };
        if let Some(irregulars) = new_candidates {
            candidates.extend(irregulars.iter().map(|c| c.to_string()));
        }

        // Just add the ending: smart -> smarter/smartest
        self.add_valid_candidate(&mut candidates, format!("{}{}", adj_str, ending));

        // Double consonant: big -> bigger/biggest
        let penult = adj_chars[adj_chars.len() - 2];
        let last = adj_chars[adj_chars.len() - 1];
        if VOWELS.contains(&penult) && !VOWELS.contains(&last) {
            self.add_valid_candidate(&mut candidates, format!("{}{}{}", adj_str, last, ending));
        }

        if last == 'y' {
            // smelly -> smellier/smelliest
            self.add_valid_candidate(
                &mut candidates,
                format!(
                    "{}i{}",
                    &adj_chars[0..adj_chars.len() - 1].iter().collect::<String>(),
                    ending
                ),
            );
        } else if last == 'e' {
            // cute -> cuter/cutest
            self.add_valid_candidate(
                &mut candidates,
                format!(
                    "{}{}",
                    &adj_chars[0..adj_chars.len() - 1].iter().collect::<String>(),
                    ending
                ),
            );
        }

        if candidates.is_empty() {
            return None;
        }

        let suggestions = candidates
            .iter()
            .map(|c| {
                Suggestion::replace_with_match_case(
                    c.chars().collect_vec(),
                    phrase.get_content(src),
                )
            })
            .collect::<Vec<Suggestion>>();

        Some(Lint {
            span: phrase,
            // Not `LintKind::Style` or `LintKind::Usage` since those can imply that the inflected form
            // is always preferred over the two-word phrase. `LintKind::WordChoice` leaves it up to you.
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "This is not an error, but an inflected form of this adjective also exists"
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for comparative adjective constructions with `more` than could use inflected forms."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };
    use crate::spell::FstDictionary;

    // True positives

    #[test]
    fn add_er() {
        assert_suggestion_result(
            "The red car is more fast.",
            MoreAdjective::new(FstDictionary::curated()),
            "The red car is faster.",
        );
    }

    #[test]
    fn add_r() {
        assert_suggestion_result(
            "The fluffy one is more cute.",
            MoreAdjective::new(FstDictionary::curated()),
            "The fluffy one is cuter.",
        );
    }

    #[test]
    fn double_final_consonant() {
        assert_suggestion_result(
            "You'll find out when you're more big.",
            MoreAdjective::new(FstDictionary::curated()),
            "You'll find out when you're bigger.",
        )
    }

    #[test]
    fn final_y() {
        assert_suggestion_result(
            "That one was even more smelly!",
            MoreAdjective::new(FstDictionary::curated()),
            "That one was even smellier!",
        );
    }

    #[test]
    fn irregular_good() {
        assert_suggestion_result(
            "I bet you couldn't do more good.",
            MoreAdjective::new(FstDictionary::curated()),
            "I bet you couldn't do better.",
        );
    }

    #[test]
    fn irregular_far() {
        assert_good_and_bad_suggestions(
            "Is it much more far?",
            MoreAdjective::new(FstDictionary::curated()),
            &["Is it much further?", "Is it much farther?"],
            &[],
        );
    }

    #[test]
    fn humane() {
        assert_suggestion_result(
            "That Klingon is more humane than the humans!",
            MoreAdjective::new(FstDictionary::curated()),
            "That Klingon is humaner than the humans!",
        );
    }

    // False positives

    #[test]
    fn dont_flag_more_time() {
        assert_no_lints(
            "I need more time.",
            MoreAdjective::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_more_model() {
        assert_no_lints(
            "Expanded access to more model architectures",
            MoreAdjective::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_more_human() {
        assert_no_lints(
            "I am more human than machine.",
            MoreAdjective::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_more_battle() {
        assert_no_lints(
            "and has more battle-tested defaults",
            MoreAdjective::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_more_like() {
        assert_no_lints(
            "It's more like a suggestion than a mistake.",
            MoreAdjective::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_more_ground() {
        assert_no_lints(
            "This E2E security scan covers more ground",
            MoreAdjective::new(FstDictionary::curated()),
        );
    }
}

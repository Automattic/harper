use std::sync::Arc;

use hashbrown::HashSet;

use crate::expr::Expr;
use crate::linting::{
    ExprLinter, LintKind, Suggestion,
    expr_linter::{Chunk, at_start_of_sentence, preceded_by_word},
};
use crate::spell::{Dictionary, FstDictionary, TrieDictionary};
use crate::{Lint, Token};

pub struct SplitWords {
    dict: Arc<TrieDictionary<Arc<FstDictionary>>>,
    expr: Box<dyn Expr>,
}

impl SplitWords {
    pub fn new() -> Self {
        Self {
            dict: TrieDictionary::curated(),
            expr: Box::new(|tok: &Token, _: &[char]| tok.kind.is_word()),
        }
    }
}

impl Default for SplitWords {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for SplitWords {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Finds missing spaces in improper compound words."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let word = &matched_tokens[0];

        // If it's a recognized word, we don't care about it.
        if word.kind.as_word().unwrap().is_some() {
            return None;
        }

        let chars = &word.get_ch(source);

        // Get all possible prefix candidates from trie and extract valid split positions
        let candidates = self.dict.find_words_with_common_prefix(chars);
        let len = chars.len();
        let mut valid_positions: HashSet<usize> = HashSet::new();

        for candidate in candidates {
            if candidate.len() >= len {
                continue;
            }
            valid_positions.insert(candidate.len());
        }

        // Generate middle-outward position order based on heuristic from PR #885:
        // Missing spaces are more likely near the middle of a word
        let mid = len / 2;
        let mut positions: Vec<usize> = Vec::new();
        positions.push(mid);

        for offset in 1..len {
            if mid >= offset {
                positions.push(mid - offset);
            }
            if mid + offset < len {
                positions.push(mid + offset);
            }
        }

        let mut suggestions = Vec::new();
        let mut has_anchor_split = false;
        let mut message: Option<String> = None;

        // Check positions in middle-outward order
        for split_pos in positions {
            if split_pos == 0 || split_pos >= len || !valid_positions.contains(&split_pos) {
                continue;
            }

            let candidate = &chars[..split_pos];
            let remainder = &chars[split_pos..];

            // Both parts must be valid common words
            let Some(cand_meta) = self.dict.get_word_metadata(candidate) else {
                continue;
            };
            if !cand_meta.common {
                continue;
            }

            let Some(rem_meta) = self.dict.get_word_metadata(remainder) else {
                continue;
            };
            if !rem_meta.common {
                continue;
            }

            if is_anchor_split(&cand_meta, candidate) || is_anchor_split(&rem_meta, remainder) {
                has_anchor_split = true;
            }

            // Valid split found
            let mut suggestion = Vec::new();
            suggestion.extend(candidate.iter());
            suggestion.push(' ');
            suggestion.extend(remainder.iter());

            suggestions.push(Suggestion::ReplaceWith(suggestion));
            if suggestions.len() == 1 {
                message = Some(format!(
                    "`{}` should probably be written as `{} {}`.",
                    chars.iter().collect::<String>(),
                    candidate.iter().collect::<String>(),
                    remainder.iter().collect::<String>()
                ));
            }
        }

        if !suggestions.is_empty() {
            let original_word: String = chars.iter().collect();

            if should_defer_to_spellcheck(&self.dict, chars, has_anchor_split, context) {
                return None;
            }

            if suggestions.len() != 1 {
                message = Some(format!(
                    "`{original_word}` has a missing space between words."
                ));
            }

            return Some(Lint {
                span: word.span,
                lint_kind: LintKind::Typo,
                suggestions,
                message: message?,
                priority: 31,
            });
        }

        None
    }
}

fn is_anchor_split(meta: &crate::DictWordMetadata, word: &[char]) -> bool {
    meta.preposition
        || meta.is_determiner()
        || meta.is_conjunction()
        || meta.is_pronoun()
        || meta.is_adverb()
        || word.len() <= 2
}

fn should_defer_to_spellcheck(
    dict: &TrieDictionary<Arc<FstDictionary>>,
    chars: &[char],
    has_anchor_split: bool,
    context: Option<(&[Token], &[Token])>,
) -> bool {
    if has_anchor_split {
        return false;
    }

    let nounish_context = context.is_some_and(|_| {
        at_start_of_sentence(context)
            || preceded_by_word(context, |tok| {
                tok.kind.is_determiner()
                    || tok.kind.is_pronoun()
                    || tok.kind.is_adjective()
                    || tok.kind.is_possessive_determiner()
            })
    });

    if !nounish_context {
        return false;
    }

    // If the whole word has a strong one-word correction, prefer that over a content-word split.
    dict.fuzzy_match(chars, 1, 1)
        .first()
        .is_some_and(|suggestion| suggestion.edit_distance == 1)
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::SplitWords;

    #[test]
    fn issue_1905() {
        assert_suggestion_result(
            "I want to try this insteadof that.",
            SplitWords::default(),
            "I want to try this instead of that.",
        );
    }

    /// Same as above, but with the longer component word at the end.
    #[test]
    fn issue_1905_rev() {
        assert_suggestion_result(
            "I want to try thisinstead of that.",
            SplitWords::default(),
            "I want to try this instead of that.",
        );
    }

    #[test]
    fn split_common() {
        assert_suggestion_result(
            "This is notnot a problem.",
            SplitWords::default(),
            "This is not not a problem.",
        );
    }

    #[test]
    fn splits_multiple_compound_words() {
        assert_suggestion_result(
            "We stared intothe darkness and kindof panicked about sortof everything.",
            SplitWords::default(),
            "We stared into the darkness and kind of panicked about sort of everything.",
        );
    }

    #[test]
    fn splits_word_with_longer_prefix() {
        assert_suggestion_result(
            "The astronauts waited on the landingpad for hours.",
            SplitWords::default(),
            "The astronauts waited on the landing pad for hours.",
        );
    }

    #[test]
    fn splits_before_punctuation() {
        assert_suggestion_result(
            "This was kindof, actually, hilarious.",
            SplitWords::default(),
            "This was kind of, actually, hilarious.",
        );
    }

    #[test]
    fn ignores_known_compound_words() {
        assert_no_lints("Someone left early.", SplitWords::default());
    }

    #[test]
    fn ignores_prefix_without_valid_remainder() {
        assert_no_lints("The monkeyxyz escaped unnoticed.", SplitWords::default());
    }

    #[test]
    fn ignores_single_word_misspelling_with_split_like_halves() {
        assert_no_lints("I love this extention!", SplitWords::default());
    }

    #[test]
    fn corrects_doesthe() {
        assert_suggestion_result("doesthe", SplitWords::default(), "does the");
    }

    #[test]
    fn corrects_splitwords() {
        assert_suggestion_result("splitwords", SplitWords::default(), "split words");
    }

    #[test]
    fn test_atall_to_at_all() {
        assert_suggestion_result(
            "don't seem to support symbolic links atall.",
            SplitWords::default(),
            "don't seem to support symbolic links at all.",
        );
    }

    #[test]
    fn test_atall_to_a_tall() {
        assert_suggestion_result("atall", SplitWords::default(), "a tall");
    }

    #[test]
    fn atall_should_split_to_a_tall_and_at_all() {
        assert_good_and_bad_suggestions("atall", SplitWords::default(), &["a tall", "at all"], &[]);
    }

    #[test]
    fn issue_2763_leaves() {
        assert_suggestion_result(
            "I love to eat cornleaves.",
            SplitWords::default(),
            "I love to eat corn leaves.",
        );
    }

    #[test]
    fn issue_2763_husks() {
        assert_suggestion_result(
            "I love to eat cornhusks.",
            SplitWords::default(),
            "I love to eat corn husks.",
        );
    }

    #[test]
    fn issue_2763_singular() {
        assert_suggestion_result(
            "I would love to eat a cornleaf.",
            SplitWords::default(),
            "I would love to eat a corn leaf.",
        );
    }
}

use std::collections::BTreeSet;

use crate::{
    Document, Punctuation, Span, TokenKind,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Flags clusters of punctuation that should be collapsed to a single mark
/// (e.g. `!!`, `?!?`, `//`, `.,`, `; :`, etc.).
#[derive(Debug, Default)]
pub struct PunctuationClusters;

impl PunctuationClusters {
    /// Punctuation kinds we’re willing to condense.
    fn is_candidate(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Punctuation(
                Punctuation::Comma
                    | Punctuation::Semicolon
                    | Punctuation::Colon
                    | Punctuation::ForwardSlash
                    | Punctuation::Bang
                    | Punctuation::Question
                    | Punctuation::Period
                    | Punctuation::Ampersand
            )
        )
    }

    /// Map a candidate punctuation token to its canonical char.
    fn char_of(kind: &TokenKind) -> char {
        match kind {
            TokenKind::Punctuation(Punctuation::Comma) => ',',
            TokenKind::Punctuation(Punctuation::Semicolon) => ';',
            TokenKind::Punctuation(Punctuation::Colon) => ':',
            TokenKind::Punctuation(Punctuation::ForwardSlash) => '/',
            TokenKind::Punctuation(Punctuation::Bang) => '!',
            TokenKind::Punctuation(Punctuation::Question) => '?',
            TokenKind::Punctuation(Punctuation::Period) => '.',
            TokenKind::Punctuation(Punctuation::Ampersand) => '&',
            _ => unreachable!("`char_of` called on non-candidate punctuation"),
        }
    }
}

impl Linter for PunctuationClusters {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let toks = document.get_tokens();
        let mut lints = Vec::new();
        let mut i = 0;
        let mut uniq = BTreeSet::<char>::new();

        while i < toks.len() {
            if !Self::is_candidate(&toks[i].kind) {
                i += 1;
                continue;
            }

            let start = i;
            let mut last_punct = i; // last index that is a candidate punctuation
            uniq.clear();
            uniq.insert(Self::char_of(&toks[i].kind));

            // scan forward, only consuming spaces if they are *between* punctuation
            let mut j = i + 1;
            while j < toks.len() {
                match &toks[j].kind {
                    k if Self::is_candidate(k) => {
                        uniq.insert(Self::char_of(k));
                        last_punct = j;
                        j += 1;
                    }
                    TokenKind::Space(_) | TokenKind::Newline(_) => {
                        // Peek past contiguous whitespace.
                        let mut k = j;
                        while k < toks.len()
                            && matches!(toks[k].kind, TokenKind::Space(_) | TokenKind::Newline(_))
                        {
                            k += 1;
                        }
                        // If whitespace is followed by more punctuation, it's *internal* → consume it.
                        if k < toks.len() && Self::is_candidate(&toks[k].kind) {
                            uniq.insert(Self::char_of(&toks[k].kind));
                            last_punct = k;
                            j = k + 1;
                        } else {
                            // Whitespace is *external* (before a word/newline/end) → stop BEFORE it.
                            break;
                        }
                    }
                    _ => break,
                }
            }

            // Count punctuation within the cluster range (no external trailing whitespace).
            let count = (start..=last_punct)
                .filter(|idx| Self::is_candidate(&toks[*idx].kind))
                .count();

            if count >= 2 {
                let span = Span::new(toks[start].span.start, toks[last_punct].span.end);

                // One suggestion per distinct glyph, external whitespace preserved outside span.
                let suggestions = uniq
                    .iter()
                    .map(|c| Suggestion::ReplaceWith(vec![*c]))
                    .collect::<Vec<_>>();

                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Formatting,
                    suggestions,
                    message: "Condense this punctuation cluster to a single mark.".into(),
                    priority: 63,
                });

                i = last_punct + 1;
            } else {
                i += 1;
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects consecutive or mixed punctuation marks that should be reduced \
         to a single comma, semicolon, colon, slash, question mark, \
         exclamation mark, period, or ampersand."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_suggestion_result, assert_top3_suggestion_result};

    use super::PunctuationClusters;

    #[test]
    fn flags_double_comma() {
        assert_suggestion_result(
            "Wait,, what happened?",
            PunctuationClusters,
            "Wait, what happened?",
        );
    }

    #[test]
    fn flags_double_semicolon() {
        assert_suggestion_result(
            "He hesitated;; then spoke.",
            PunctuationClusters,
            "He hesitated; then spoke.",
        );
    }

    #[test]
    fn flags_double_colon() {
        assert_suggestion_result("Choices:: A or B.", PunctuationClusters, "Choices: A or B.");
    }

    #[test]
    fn flags_double_bang() {
        assert_suggestion_result("Stop!!", PunctuationClusters, "Stop!");
    }

    #[test]
    fn flags_double_question() {
        assert_suggestion_result("Really??", PunctuationClusters, "Really?");
    }

    #[test]
    fn flags_mixed_qbang_pair() {
        assert_top3_suggestion_result("What?!", PunctuationClusters, "What?");
    }

    #[test]
    fn flags_triple_bang() {
        assert_suggestion_result(
            "No!!! Absolutely not.",
            PunctuationClusters,
            "No! Absolutely not.",
        );
    }

    #[test]
    fn flags_q_bang_bang() {
        assert_top3_suggestion_result("Really?!!", PunctuationClusters, "Really?");
    }

    #[test]
    fn flags_double_slash() {
        assert_suggestion_result(
            "This // is a typo.",
            PunctuationClusters,
            "This / is a typo.",
        );
    }

    #[test]
    fn flags_triple_question() {
        assert_suggestion_result("Why???", PunctuationClusters, "Why?");
    }

    #[test]
    fn flags_quadruple_bang() {
        assert_suggestion_result("Stop!!!!", PunctuationClusters, "Stop!");
    }

    #[test]
    fn flags_question_bang_question() {
        assert_top3_suggestion_result("You did what?!?", PunctuationClusters, "You did what?");
    }

    #[test]
    fn flags_bang_question_bang() {
        assert_top3_suggestion_result("No way!?!", PunctuationClusters, "No way!");
    }

    #[test]
    fn flags_question_bang_bang_question() {
        assert_top3_suggestion_result("Seriously?!!?", PunctuationClusters, "Seriously?");
    }

    #[test]
    fn flags_with_intervening_whitespace() {
        assert_top3_suggestion_result("Why?! ?", PunctuationClusters, "Why?");
    }

    #[test]
    fn flags_double_ampersand() {
        assert_suggestion_result("This && that.", PunctuationClusters, "This & that.");
    }

    #[test]
    fn flags_period_comma_cluster() {
        assert_top3_suggestion_result("Oops., excuse me.", PunctuationClusters, "Oops, excuse me.");
    }

    #[test]
    fn flags_colon_comma_cluster() {
        assert_top3_suggestion_result(
            "Delay:, we must wait.",
            PunctuationClusters,
            "Delay: we must wait.",
        );
    }

    #[test]
    fn flags_semicolon_colon_cluster() {
        assert_top3_suggestion_result("Choices;: A or B.", PunctuationClusters, "Choices: A or B.");
    }

    #[test]
    fn flags_comma_period_cluster() {
        assert_top3_suggestion_result(
            "Hold on,. actually…",
            PunctuationClusters,
            "Hold on, actually…",
        );
    }

    #[test]
    fn flags_question_period_cluster() {
        assert_top3_suggestion_result("Really?.", PunctuationClusters, "Really?");
        assert_top3_suggestion_result("Really?.", PunctuationClusters, "Really.");
    }

    #[test]
    fn flags_bang_period_cluster() {
        assert_suggestion_result("Stop!.", PunctuationClusters, "Stop!");
    }
}

use super::{Lint, LintKind, Linter, Suggestion};
use crate::document::Document;
use crate::{Punctuation, Token, TokenKind, TokenStringExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedCase {
    Capital,
    Lowercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interruption {
    None,
    Attribution,
    Bare,
    ClosedSentence,
    NewSpeech,
    Clause,
}

/// A linter that checks that quoted dialogue and its attributions are capitalized
/// correctly.
#[derive(Debug, Clone, Copy, Default)]
pub struct DialogueCapitalization;

impl Linter for DialogueCapitalization {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let tokens = document.get_tokens();
        let mut lints = Vec::new();

        let source = document.get_source();

        for quote_idx in tokens.iter_quote_indices() {
            let expectation = if is_opening_quote(source, tokens, quote_idx) {
                if starts_with_continuation_marker(tokens, quote_idx) {
                    None
                } else {
                    dialogue_expectation(tokens, quote_idx)
                }
            } else if is_closing_quote(source, tokens, quote_idx) {
                attribution_expectation(tokens, quote_idx)
            } else {
                None
            };

            let Some(expected) = expectation else {
                continue;
            };

            let Some(word_idx) = first_word_after(tokens, quote_idx) else {
                continue;
            };

            push_lint_if_miscased(document, &mut lints, word_idx, expected);
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Checks that quoted dialogue and its attributions are capitalized correctly."
    }
}

fn push_lint_if_miscased(
    document: &Document,
    lints: &mut Vec<Lint>,
    word_idx: usize,
    expected: ExpectedCase,
) {
    let word = &document.get_tokens()[word_idx];
    let word_chars = document.get_span_content(&word.span);

    let Some(first_char) = word_chars.first().copied() else {
        return;
    };

    // Unknown words may be names or jargon.
    if matches!(word.kind, TokenKind::Word(None)) {
        return;
    }

    // Proper nouns and brands keep their capitalization. Pronouns are excluded
    // because the dictionary marks `He` as a proper noun.
    let is_proper_noun = word.kind.is_proper_noun() && !word.kind.is_pronoun();

    if is_proper_noun || word_chars.iter().skip(1).any(|c| c.is_uppercase()) {
        return;
    }

    let (message, replacement_char) = match expected {
        ExpectedCase::Capital => {
            if !first_char.is_lowercase() {
                return;
            }

            (
                "This starts a sentence, so it should be capitalized.",
                first_char.to_uppercase().next(),
            )
        }
        ExpectedCase::Lowercase => {
            if !first_char.is_uppercase() || is_capitalized_pronoun_i(word_chars) {
                return;
            }

            (
                "This continues the interrupted sentence, so it should be lowercase.",
                first_char.to_lowercase().next(),
            )
        }
    };

    let Some(replacement_char) = replacement_char else {
        return;
    };

    let mut replacement = word_chars.to_vec();
    replacement[0] = replacement_char;

    lints.push(Lint {
        span: word.span,
        lint_kind: LintKind::Capitalization,
        suggestions: vec![Suggestion::ReplaceWith(replacement)],
        message: message.to_owned(),
        priority: 31,
    });
}

fn is_capitalized_pronoun_i(word_chars: &[char]) -> bool {
    match word_chars {
        ['I'] => true,
        ['I', next, ..] => matches!(*next, '\'' | '’'),
        _ => false,
    }
}

fn starts_with_continuation_marker(tokens: &[Token], quote_idx: usize) -> bool {
    tokens
        .iter()
        .skip(quote_idx + 1)
        .take_while(|tok| !tok.kind.is_word())
        .any(|tok| {
            let kind = &tok.kind;

            kind.is_quote()
                || kind.is_open_single()
                || matches!(
                    kind,
                    TokenKind::Punctuation(Punctuation::EmDash)
                        | TokenKind::Punctuation(Punctuation::EnDash)
                )
        })
}

/// Whether the quote at `idx` opens a quoted segment.
fn is_opening_quote(source: &[char], tokens: &[Token], idx: usize) -> bool {
    let Some(quote) = tokens[idx].kind.as_quote() else {
        return false;
    };

    // Curly quotes point one way, so trust them over pairing.
    match tokens[idx].get_ch(source).first() {
        Some('“') => return true,
        Some('”') => return false,
        _ => {}
    }

    if let Some(twin) = quote.twin_loc {
        return twin > idx;
    }

    match previous_non_whitespace(tokens, idx) {
        Some(prev) => !tokens[prev].kind.is_word_like(),
        None => true,
    }
}

/// Whether the quote at `idx` closes a quoted segment.
fn is_closing_quote(source: &[char], tokens: &[Token], idx: usize) -> bool {
    let Some(quote) = tokens[idx].kind.as_quote() else {
        return false;
    };

    match tokens[idx].get_ch(source).first() {
        Some('“') => return false,
        Some('”') => return true,
        _ => {}
    }

    quote.twin_loc.is_some_and(|twin| twin < idx)
}

fn previous_non_whitespace(tokens: &[Token], idx: usize) -> Option<usize> {
    tokens[..idx]
        .iter()
        .rposition(|tok| !tok.kind.is_whitespace())
}

fn next_non_whitespace(tokens: &[Token], idx: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(idx + 1)
        .find(|(_, tok)| !tok.kind.is_whitespace())
        .map(|(idx, _)| idx)
}

fn first_word_after(tokens: &[Token], quote_idx: usize) -> Option<usize> {
    let mut punctuation_skipped = 0;

    for (idx, tok) in tokens.iter().enumerate().skip(quote_idx + 1) {
        if tok.kind.is_whitespace() {
            continue;
        }

        if tok.kind.is_word() {
            return Some(idx);
        }

        if tok.kind.is_punctuation() && !tok.kind.is_quote() && punctuation_skipped < 2 {
            punctuation_skipped += 1;
            continue;
        }

        return None;
    }

    None
}

fn dialogue_expectation(tokens: &[Token], quote_idx: usize) -> Option<ExpectedCase> {
    let Some(prev) = previous_non_whitespace(tokens, quote_idx) else {
        return Some(ExpectedCase::Capital);
    };

    let prev_kind = &tokens[prev].kind;

    if prev_kind.is_sentence_terminator() {
        return Some(ExpectedCase::Capital);
    }

    if prev_kind.is_comma() {
        return match interruption_before(tokens, quote_idx) {
            // The attribution interrupted the dialogue.
            Interruption::Attribution => Some(ExpectedCase::Lowercase),
            // A comma plus a full quoted sentence starts new dialogue.
            Interruption::None | Interruption::NewSpeech if ends_sentence(tokens, quote_idx) => {
                Some(ExpectedCase::Capital)
            }
            // Quoted list items and new clauses are ambiguous.
            _ => None,
        };
    }

    // An em dash that interrupts an attribution.
    if matches!(prev_kind, TokenKind::Punctuation(Punctuation::EmDash)) {
        return match interruption_before(tokens, quote_idx) {
            Interruption::Attribution => Some(ExpectedCase::Lowercase),
            _ => None,
        };
    }

    None
}

fn ends_sentence(tokens: &[Token], open_idx: usize) -> bool {
    let Some(close_idx) = tokens[open_idx]
        .kind
        .as_quote()
        .and_then(|quote| quote.twin_loc)
    else {
        return false;
    };

    for (idx, tok) in tokens.iter().enumerate().skip(close_idx + 1) {
        if tok.kind.is_space() {
            continue;
        }

        if tok.kind.is_sentence_terminator() {
            return true;
        }

        // A hard-wrapped line continues the same sentence.
        let ends_the_line =
            tok.kind.is_paragraph_break() || matches!(tok.kind, TokenKind::Newline(_));

        if ends_the_line {
            let inside_terminator = previous_non_whitespace(tokens, close_idx)
                .is_some_and(|prev| tokens[prev].kind.is_sentence_terminator());

            if !inside_terminator {
                return false;
            }

            return match next_non_whitespace(tokens, idx) {
                Some(next) => tokens[next].kind.is_quote(),
                None => true,
            };
        }

        return false;
    }

    true
}

fn attribution_expectation(tokens: &[Token], close_idx: usize) -> Option<ExpectedCase> {
    let prev = previous_non_whitespace(tokens, close_idx)?;
    let prev_kind = &tokens[prev].kind;

    if prev_kind.is_comma() {
        return Some(ExpectedCase::Lowercase);
    }

    if prev_kind.is_period() {
        return Some(ExpectedCase::Capital);
    }

    // After `?` or `!`, the next words could be an attribution or an action beat.
    None
}

fn interruption_before(tokens: &[Token], quote_idx: usize) -> Interruption {
    let mut saw_word = false;
    let mut saw_newline = false;
    let mut saw_clause_break = false;

    for (idx, tok) in tokens[..quote_idx].iter().enumerate().rev() {
        if tok.kind.is_sentence_terminator() {
            break;
        }

        // A conjunction or separator means a new clause, not a continuation.
        if tok.kind.is_conjunction()
            || tok.kind.is_semicolon()
            || matches!(tok.kind, TokenKind::Punctuation(Punctuation::Colon))
        {
            saw_clause_break = true;
        }

        if tok.kind.is_quote() {
            let closes_sentence = previous_non_whitespace(tokens, idx)
                .is_some_and(|prev| tokens[prev].kind.is_sentence_terminator());

            if closes_sentence {
                return if saw_newline {
                    Interruption::NewSpeech
                } else {
                    Interruption::ClosedSentence
                };
            }

            return match (saw_word, saw_clause_break) {
                (false, _) => Interruption::Bare,
                (true, false) => Interruption::Attribution,
                (true, true) => Interruption::Clause,
            };
        }

        if matches!(tok.kind, TokenKind::Newline(_)) {
            saw_newline = true;
        } else if tok.kind.is_word() {
            saw_word = true;
        }
    }

    Interruption::None
}

#[cfg(test)]
mod tests {
    use super::DialogueCapitalization;
    use crate::linting::Linter;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    // Failing examples from `dev-examples.txt`.

    #[test]
    fn capitalizes_dialogue_after_interrupting_attribution() {
        assert_lint_count(
            "“Dialogue starts but interrupted,” he said, doing, “Dialogue ends.”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn lowercase_suggestion_after_interrupting_attribution() {
        assert_suggestion_result(
            "“Dialogue starts but interrupted,” he said, doing, “Dialogue ends.”",
            DialogueCapitalization,
            "“Dialogue starts but interrupted,” he said, doing, “dialogue ends.”",
        );
    }

    #[test]
    fn capitalizes_dialogue_after_attribution_ending_in_period() {
        assert_lint_count(
            "“Dialogue starts but interrupted,” he said, doing. “dialogue ends.”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn capitalizes_dialogue_after_attribution() {
        assert_lint_count("He said, “dialogue starts.”", DialogueCapitalization, 1);
    }

    #[test]
    fn capitalize_suggestion_after_attribution() {
        assert_suggestion_result(
            "He said, “dialogue starts.”",
            DialogueCapitalization,
            "He said, “Dialogue starts.”",
        );
    }

    #[test]
    fn lowercases_dialogue_after_interrupting_attribution() {
        assert_lint_count(
            "“Dialogue starts,” he said, “Dialogue ends.”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn capitalizes_dialogue_after_attribution_sentence() {
        assert_lint_count(
            "“Dialogue starts,” he said. “dialogue ends.”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn lowercases_attribution_after_comma() {
        assert_lint_count(
            "“Dialogue starts,” He said. He performs an action.",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn lowercase_attribution_suggestion() {
        assert_suggestion_result(
            "“Dialogue starts,” He said.",
            DialogueCapitalization,
            "“Dialogue starts,” he said.",
        );
    }

    #[test]
    fn capitalizes_attribution_after_period() {
        assert_lint_count(
            "“Dialogue starts.” he said. He performs an action.",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn lowercases_dialogue_after_em_dash_interruption() {
        assert_lint_count(
            "“Dialogue starts,”—he interrupts with an action—“Dialogue ends.”",
            DialogueCapitalization,
            1,
        );
    }

    // Allowed examples from `dev-examples.txt`.

    #[test]
    fn allows_lowercase_dialogue_after_interrupting_attribution() {
        assert_no_lints(
            "“Dialogue starts,” he said, “dialogue ends.”",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_capitalized_dialogue_after_attribution_sentence() {
        assert_no_lints(
            "“Dialogue starts,” he said. “Dialogue ends.”",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_capitalized_attribution_after_period() {
        assert_no_lints(
            "“Dialogue starts.” He said. He performs an action.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_lowercase_attribution_after_comma() {
        assert_no_lints(
            "“Dialogue starts,” he said. He performs an action.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_lowercase_dialogue_after_em_dash_interruption() {
        assert_no_lints(
            "“Dialogue starts,”—he interrupts with an action—“dialogue ends.”",
            DialogueCapitalization,
        );
    }

    #[test]
    fn catches_ascii_quotes() {
        assert_lint_count(
            "She whispered, \"we should leave now.\"",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn catches_dialogue_at_paragraph_start() {
        assert_lint_count("“dialogue starts,” he said.", DialogueCapitalization, 1);
    }

    #[test]
    fn catches_dialogue_after_sentence_in_narration() {
        assert_lint_count(
            "The teacher waited. “silence, please.”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn catches_both_problems_at_once() {
        assert_lint_count(
            "“dialogue starts,” he said, doing, “Dialogue ends.”",
            DialogueCapitalization,
            2,
        );
    }

    #[test]
    fn allows_proper_nouns() {
        assert_no_lints("He said, “Bobby is here.”", DialogueCapitalization);
    }

    #[test]
    fn allows_proper_noun_attributions() {
        assert_no_lints("“Dialogue starts,” John said.", DialogueCapitalization);
    }

    #[test]
    fn allows_pronoun_i_in_attributions() {
        assert_no_lints("“Dialogue starts,” I said.", DialogueCapitalization);
    }

    #[test]
    fn allows_acronyms() {
        assert_no_lints("He said, “NASA is coming.”", DialogueCapitalization);
    }

    #[test]
    fn allows_camel_case_brands() {
        assert_no_lints("He said, “iPhone sales are up.”", DialogueCapitalization);
    }

    #[test]
    fn allows_mid_sentence_quotes() {
        assert_no_lints("The word “dialogue” is a noun.", DialogueCapitalization);
    }

    #[test]
    fn allows_quoted_lists() {
        assert_no_lints(
            "The choices are low, “high”, and “medium”.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn catches_present_tense_attribution() {
        assert_lint_count("She whispers, “hello.”", DialogueCapitalization, 1);
    }

    #[test]
    fn catches_progressive_attribution() {
        assert_lint_count("He is whispering, “hello.”", DialogueCapitalization, 1);
    }

    #[test]
    fn catches_attribution_ending_in_ies() {
        assert_lint_count("She replies, “hello.”", DialogueCapitalization, 1);
    }

    #[test]
    fn catches_attribution_with_indirect_object() {
        assert_lint_count("She told him, “we are late.”", DialogueCapitalization, 1);
    }

    #[test]
    fn catches_question_ending_dialogue_after_attribution() {
        assert_lint_count("He asked, “where is it?”", DialogueCapitalization, 1);
    }

    #[test]
    fn catches_capitalized_attribution_between_quotes() {
        assert_lint_count(
            "“Dialogue,” She said, “dialogue.”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn catches_capitalized_continuation_after_asked() {
        assert_lint_count(
            "“Hello,” she said, “Where are you?”",
            DialogueCapitalization,
            1,
        );
    }

    #[test]
    fn allows_dialogue_with_multiple_sentences() {
        assert_no_lints(
            "“I'm here. Where are you?” she asked.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_short_continuation() {
        assert_no_lints("“Wait,” she said, “what?”", DialogueCapitalization);
    }

    #[test]
    fn allows_quote_without_preceding_punctuation() {
        assert_no_lints("He said “hello” and left.", DialogueCapitalization);
    }

    #[test]
    fn allows_em_dash_cutoff() {
        assert_no_lints("“I can't—” he started.", DialogueCapitalization);
    }

    #[test]
    fn allows_quoted_titles() {
        assert_no_lints("Please read “The Hobbit” tonight.", DialogueCapitalization);
    }

    #[test]
    fn allows_action_beat_after_dialogue() {
        assert_no_lints("“Stop!” She slammed the door.", DialogueCapitalization);
    }

    #[test]
    fn allows_consecutive_quotes() {
        assert_no_lints("“Hi.” “Hello.”", DialogueCapitalization);
    }

    #[test]
    fn allows_lowercase_narration_after_attribution() {
        assert_no_lints(
            "“Hello,” he said, and then he left.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_trailing_quoted_comma() {
        assert_no_lints("He said, “Hello,” and left.", DialogueCapitalization);
    }

    #[test]
    fn allows_ellipsis_at_end_of_dialogue() {
        assert_no_lints("“I don't know…” she said.", DialogueCapitalization);
    }

    #[test]
    fn allows_quoted_terms_after_colon() {
        assert_no_lints("The label reads: “fragile”.", DialogueCapitalization);
    }

    #[test]
    fn allows_quoted_terms_after_comma() {
        assert_no_lints("The word, “dialogue,” is a noun.", DialogueCapitalization);
    }

    #[test]
    fn allows_quoted_fragment_followed_by_more_text() {
        assert_no_lints(
            "It caused issues with phrases like, \"it was Mike Tyson vs. Weird Al!\" and more.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_quoted_appositive_followed_by_more_text() {
        assert_no_lints(
            "The sign read, “closed,” which surprised me.",
            DialogueCapitalization,
        );
    }

    #[test]
    fn allows_attribution_after_exclamation() {
        assert_no_lints("“Stop!” she yelled.", DialogueCapitalization);
    }

    #[test]
    fn catches_dialogue_on_the_next_line() {
        let source = concat!(
            "“Dialogue starts but interrupted,” he said, doing. “dialogue ends.”\n",
            "He said, “dialogue starts.”"
        );
        let document = crate::Document::new_markdown_default_curated(source);

        assert_eq!(DialogueCapitalization.lint(&document).len(), 2);
    }
}

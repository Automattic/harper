use crate::char_string::CharStringExt;
use crate::patterns::WhitespacePattern;
use crate::{
    Token, TokenKind,
    expr::{AnchorEnd, Expr, FirstMatchOf, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToToo {
    expr: Box<dyn Expr>,
}

impl Default for ToToo {
    fn default() -> Self {
        // Ported from linting/to_too.rs
        let to_before_adj_adv = SequenceExpr::default().t_aco("to").t_ws().then_any_of(vec![
            // to + adjective (but not also a verb) + (punctuation | end)
            // This avoids flagging prepositional "to" when followed by a noun, e.g.,
            // "to traditional solutions".
            Box::new(
                SequenceExpr::default()
                    .then_adjective()
                    .then_optional(WhitespacePattern)
                    .then_optional(SequenceExpr::default().then_any_word())
                    .then_optional(WhitespacePattern)
                    .then_optional(SequenceExpr::default().then_punctuation()),
            ),
            // to + adverb + (punctuation | end) -> avoid infinitive like "to properly generate"
            Box::new(
                SequenceExpr::default()
                    .then_adverb()
                    .then_optional(WhitespacePattern)
                    .then_any_of(vec![
                        Box::new(SequenceExpr::default().then_punctuation()),
                        Box::new(
                            SequenceExpr::default().then_unless(SequenceExpr::default().t_any()),
                        ),
                    ]),
            ),
        ]);

        let to_before_degree_words = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_word_set(&["many", "much", "few"]);

        let chunk_start_to_pause = SequenceExpr::default()
            .then(crate::expr::AnchorStart)
            .t_aco("to")
            .then_optional(WhitespacePattern)
            .then_comma();

        let pronoun_to_end = SequenceExpr::default()
            .then_pronoun()
            .t_ws()
            .t_aco("to")
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_punctuation()),
                Box::new(AnchorEnd),
            ]);

        let expr = FirstMatchOf::new(vec![
            Box::new(to_before_adj_adv),
            Box::new(to_before_degree_words),
            Box::new(chunk_start_to_pause),
            Box::new(pronoun_to_end),
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToToo {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        // Debug output to understand matching in composite rule during tests
        eprintln!(
            "[ToTwoToo::ToToo] matched tokens: {:?}",
            tokens
                .iter()
                .map(|t| t.span.get_content(source).iter().collect::<String>())
                .collect::<Vec<_>>()
        );
        let to_tok = tokens.iter().find(|t| {
            t.span
                .get_content(source)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
        })?;
        eprintln!(
            "[ToTwoToo::ToToo] to_tok: {}",
            to_tok.span.get_content(source).iter().collect::<String>()
        );

        // Decide if this match should lint based on the token following `to`.
        // Find the next non-whitespace token after `to` (if any)
        let to_index = tokens
            .iter()
            .position(|t| {
                t.span
                    .get_content(source)
                    .eq_ignore_ascii_case_chars(&['t', 'o'])
            })
            .unwrap();

        // Find index of the first non-whitespace token after `to`
        let mut idx = to_index + 1;
        while idx < tokens.len() && tokens[idx].kind.is_whitespace() {
            idx += 1;
        }

        let should_lint = if idx < tokens.len() {
            let next = &tokens[idx];
            let next_text: String = next.span.get_content(source).iter().collect();
            let next_lower = next_text.to_lowercase();
            // Find token after `next` ignoring whitespace, if any
            let mut j = idx + 1;
            while j < tokens.len() && tokens[j].kind.is_whitespace() {
                j += 1;
            }
            let after_next_non_ws = if j < tokens.len() {
                Some(&tokens[j])
            } else {
                None
            };

            // Branch: degree words
            if matches!(next_lower.as_str(), "many" | "much" | "few") {
                true
            // Branch: punctuation or end after pronoun (", to.", "Me to!")
            } else if next.kind.is_punctuation() {
                true
            // Branch: adverb
            } else if next.kind.is_adverb() {
                // Only when followed by non-quote punctuation or end-of-slice
                match after_next_non_ws {
                    None => true,
                    Some(t) => {
                        if t.kind.is_punctuation() {
                            let punct: String = t.span.get_content(source).iter().collect();
                            !matches!(punct.as_str(), "`" | "\"" | "'" | "“" | "”" | "‘" | "’")
                        } else {
                            false
                        }
                    }
                }
            // Branch: adjective (but not also a verb) + (punct | end)
            } else if next.kind.is_adjective() {
                // Do not lint before proper nouns (e.g., "to Nashville")
                if next.kind.is_proper_noun() {
                    return None;
                }
                // Avoid "to standard ..." which is commonly a prepositional phrase
                if next_lower == "standard" {
                    return None;
                }
                // Allow adjectives even if they can be verbs as long as punctuation or
                // end-of-slice follows immediately (our expr enforces this structurally).
                match after_next_non_ws {
                    None => true, // end-of-slice (no following word captured)
                    Some(t) if t.kind.is_punctuation() => {
                        let punct: String = t.span.get_content(source).iter().collect();
                        !matches!(punct.as_str(), "`" | "\"" | "'" | "“" | "”" | "‘" | "’")
                    }
                    // If a word follows, do not lint (likely "to ADJ NOUN" prepositional phrase)
                    _ => false,
                }
            } else {
                false
            }
        } else {
            // No token after `to` (end of chunk) — don't lint.
            false
        };

        eprintln!("[ToTwoToo::ToToo] should_lint={}", should_lint);
        if !should_lint {
            return None;
        }

        Some(Lint {
            span: to_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "too",
                to_tok.span.get_content(source),
            )],
            message: "Use `too` here to mean ‘also’ or an excessive degree.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects mistaken `to` to `too` when it means ‘also’ or an excessive degree."
    }
}

#[cfg(test)]
mod dbg_tests {
    use super::ToToo;
    use crate::{Document, expr::ExprExt, linting::tests::SpanVecExt};

    #[test]
    fn dbg_match_to_easy() {
        let doc = Document::new_markdown_default_curated("It's not to easy, is it?");
        let l = ToToo::default();
        let matches = l.expr.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        eprintln!("DBG matches: {:?}", matches.to_strings(&doc));
        // Print tokens and kinds
        for tok in doc.get_tokens() {
            let s = doc.get_span_content_str(&tok.span);
            eprintln!(
                "tok='{}' ws={} punct={} adj={} verb={} adv={}",
                s,
                tok.kind.is_whitespace(),
                tok.kind.is_punctuation(),
                tok.kind.is_adjective(),
                tok.kind.is_verb(),
                tok.kind.is_adverb()
            );
        }
        assert!(!matches.is_empty(), "expected at least one expr match");
    }
}

use crate::char_string::CharStringExt;
use crate::patterns::WhitespacePattern;
use crate::{
    Token, TokenKind,
    expr::{AnchorEnd, AnchorStart, Expr, FirstMatchOf, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToToo {
    expr: Box<dyn Expr>,
}

impl Default for ToToo {
    fn default() -> Self {
        // to + adjective (but not also a verb), allowing optional following word/punct.
        // Decision about linting is refined in `match_to_lint` to avoid false positives.
        let to_before_adjective_loose = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_kind_is_but_is_not_except(
                TokenKind::is_adjective,
                TokenKind::is_verb,
                &["standard"],
            )
            .then_optional(WhitespacePattern)
            .then_optional(SequenceExpr::default().then_any_word())
            .then_optional(WhitespacePattern)
            .then_optional(SequenceExpr::default().then_punctuation());

        // to + adverb + (punct | end)
        // to + adverb + (punct | end)
        let to_before_adverb = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_kind_is_but_is_not_except(
                TokenKind::is_adverb,
                |_| false,
                &["as"],
            )
            .then_optional(WhitespacePattern)
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_kind_is_but_is_not_except(
                    TokenKind::is_punctuation,
                    |_| false,
                    &["`", "\"", "'", "“", "”", "‘", "’"],
                )),
                Box::new(SequenceExpr::default().then_unless(SequenceExpr::default().t_any())),
            ]);

        // to + adjective-verb (past participle like "tired") + punctuation (non-quote, non-dash)
        // Helps catch cases like "to tired." while avoiding base verbs like "to dominate."
        let to_before_adj_verb_ed_punct = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then(|tok: &Token, src: &[char]| {
                tok.kind.is_adjective()
                    && tok.kind.is_verb()
                    && !tok.kind.is_noun()
                    && tok
                        .span
                        .get_content(src)
                        .iter()
                        .collect::<String>()
                        .to_lowercase()
                        .ends_with("ed")
            })
            .then_optional(WhitespacePattern)
            .then_kind_is_but_is_not_except(
                TokenKind::is_punctuation,
                |_| false,
                &["`", "\"", "'", "“", "”", "‘", "’", "-", "–", "—"],
            );

        // to + adjective (any, including words that can also be verbs) + punctuation (non-quote, non-dash)
        let to_before_adjective_strict_punct = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_kind_is_but_is_not_except(
                TokenKind::is_adjective,
                TokenKind::is_verb,
                &["standard"],
            )
            .then_optional(WhitespacePattern)
            .then_kind_is_but_is_not_except(
                TokenKind::is_punctuation,
                |_| false,
                &["`", "\"", "'", "“", "”", "‘", "’", "-", "–", "—"],
            );

        // to + (many|much|few) + (punct|end) to avoid "connected to many ..."
        let to_before_degree_words = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_word_set(&["many", "much", "few"])
            .t_ws_opt()
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_kind_is_but_is_not_except(
                    TokenKind::is_punctuation,
                    |_| false,
                    &["`", "\"", "'", "“", "”", "‘", "’", "-", "–", "—"],
                )),
                Box::new(AnchorEnd),
            ]);

        let chunk_start_to_pause = SequenceExpr::default()
            .then(AnchorStart)
            .t_aco("to")
            .then_optional(WhitespacePattern)
            .then_comma();

        // (start|punct) + pronoun + to + (punct_without_quotes | end)
        let pronoun_to_end = SequenceExpr::default()
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_anchor_start()),
                Box::new(
                    SequenceExpr::default()
                        .then_kind_is_but_is_not_except(
                            TokenKind::is_punctuation,
                            |_| false,
                            &["`", "\"", "'", "“", "”", "‘", "’"],
                        )
                        .t_ws_opt(),
                ),
            ])
            .then_pronoun()
            .t_ws()
            .t_aco("to")
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_kind_is_but_is_not_except(
                    TokenKind::is_punctuation,
                    |_| false,
                    &["`", "\"", "'", "“", "”", "‘", "’"],
                )),
                Box::new(AnchorEnd),
            ]);

        let expr = FirstMatchOf::new(vec![
            Box::new(to_before_adj_verb_ed_punct),
            Box::new(to_before_adjective_strict_punct),
            Box::new(to_before_adverb),
            Box::new(to_before_degree_words),
            Box::new(pronoun_to_end),
            Box::new(chunk_start_to_pause),
            Box::new(to_before_adjective_loose),
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
        // The expression ensures only valid contexts reach here.
        let to_tok = tokens.iter().find(|t| {
            t.span
                .get_content(source)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
        })?;

        // Decide if this match should lint based on the token following `to`.
        // Find the next non-whitespace token after `to` (if any)
        let to_index = tokens
            .iter()
            .position(|t| {
                t.span
                    .get_content(source)
                    .eq_ignore_ascii_case_chars(&['t', 'o'])
            })
            .unwrap_or(0);

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
                // Only when followed by non-quote, non-dash punctuation or end-of-slice
                match after_next_non_ws {
                    None => true,
                    Some(t) => {
                        if t.kind.is_punctuation() {
                            let punct: String = t.span.get_content(source).iter().collect();
                            !matches!(
                                punct.as_str(),
                                "`" | "\"" | "'" | "“" | "”" | "‘" | "’" | "-" | "–" | "—"
                            )
                        } else {
                            false
                        }
                    }
                }
            // Branch: adjective
            } else if next.kind.is_adjective() {
                // Avoid specific exception (commonly prepositional phrase)
                if next_lower == "standard" {
                    return None;
                }
                match after_next_non_ws {
                    None => true, // end-of-slice (no following token captured)
                    Some(t) if t.kind.is_punctuation() => {
                        let punct: String = t.span.get_content(source).iter().collect();
                        !matches!(
                            punct.as_str(),
                            "`" | "\"" | "'" | "“" | "”" | "‘" | "’" | "-" | "–" | "—"
                        )
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

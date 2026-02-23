use std::sync::Arc;

use super::{ExprLinter, Lint, LintKind};
use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::patterns::{ImpliesQuantity, WordSet};

pub struct ExpandMemoryShorthands {
    expr: Box<dyn Expr>,
}

impl ExpandMemoryShorthands {
    pub fn new() -> Self {
        let hotwords = Arc::new(WordSet::new(&[
            "B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB", "RB", "QB", "KiB", "MiB", "GiB",
            "TiB", "PiB", "EiB", "ZiB", "YiB", "RiB", "QiB",
        ]));

        Self {
            expr: Box::new(SequenceExpr::with(ImpliesQuantity).then_longest_of(vec![
                Box::new(SequenceExpr::with(hotwords.clone())),
                Box::new(SequenceExpr::default().t_ws_h().then(hotwords.clone())),
            ])),
        }
    }

    fn get_replacement(abbreviation: &str, plural: Option<bool>) -> Option<&'static str> {
        let is_plural = plural.unwrap_or(abbreviation.ends_with('s'));
        match abbreviation {
            "B" => Some(if is_plural { "bytes" } else { "byte" }),
            "kB" => Some(if is_plural { "kilobytes" } else { "kilobyte" }),
            "MB" => Some(if is_plural { "megabytes" } else { "megabyte" }),
            "GB" => Some(if is_plural { "gigabytes" } else { "gigabyte" }),
            "TB" => Some(if is_plural { "terabytes" } else { "terabyte" }),
            "PB" => Some(if is_plural { "petabytes" } else { "petabyte" }),
            "EB" => Some(if is_plural { "exabytes" } else { "exabyte" }),
            "ZB" => Some(if is_plural { "zettabytes" } else { "zettabyte" }),
            "YB" => Some(if is_plural { "yottabytes" } else { "yottabyte" }),
            "RB" => Some(if is_plural { "ronnabytes" } else { "ronnabyte" }),
            "QB" => Some(if is_plural {
                "quettabytes"
            } else {
                "quettabyte"
            }),
            "KiB" => Some(if is_plural { "kibibytes" } else { "kibibyte" }),
            "MiB" => Some(if is_plural { "mebibytes" } else { "mebibyte" }),
            "GiB" => Some(if is_plural { "gibibytes" } else { "gibibyte" }),
            "TiB" => Some(if is_plural { "tebibytes" } else { "tebibyte" }),
            "PiB" => Some(if is_plural { "pebibytes" } else { "pebibyte" }),
            "EiB" => Some(if is_plural { "exbibytes" } else { "exbibyte" }),
            "ZiB" => Some(if is_plural { "zebibytes" } else { "zebibyte" }),
            "YiB" => Some(if is_plural { "yobibytes" } else { "yobibyte" }),
            "RiB" => Some(if is_plural { "robibytes" } else { "robibyte" }),
            "QiB" => Some(if is_plural { "quebibytes" } else { "quebibyte" }),

            _ => None,
        }
    }
}

impl Default for ExpandMemoryShorthands {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for ExpandMemoryShorthands {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_span = matched_tokens.last()?.span;
        let implies_plural = ImpliesQuantity::implies_plurality(matched_tokens.first()?, source);

        let offending_text = offending_span.get_content(source);

        let replacement =
            Self::get_replacement(&offending_text.iter().collect::<String>(), implies_plural)?;

        let mut replacement_chars = Vec::new();

        // If there isn't spacing, insert a space
        if matched_tokens.len() == 2 {
            replacement_chars.push(' ');
        }

        replacement_chars.extend(replacement.chars());

        if replacement_chars == offending_text {
            return None;
        }

        Some(Lint {
            span: offending_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
            message: format!("Did you mean `{replacement}`?"),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Expands memory-related abbreviations (`B`, `kB`, `MB`, `GB`, `TB`, `PB`, `KiB`, `MiB`, `GiB`, `TiB`, `PiB`, etc.) to their full forms (`byte`, `kilobyte`, `megabyte`, `gigabyte`, `terabyte`, `petabyte`, `kibibyte`, `mebibyte`, `gibibyte`, `tebibyte`, `pebibyte`, etc.)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::ExpandMemoryShorthands;

    #[test]
    fn detects_bytes() {
        assert_suggestion_result("5 B", ExpandMemoryShorthands::new(), "5 bytes");
    }

    #[test]
    fn detects_kilobytes() {
        assert_suggestion_result("10 kB", ExpandMemoryShorthands::new(), "10 kilobytes");
    }

    #[test]
    fn detects_megabytes() {
        assert_suggestion_result("30 MB", ExpandMemoryShorthands::new(), "30 megabytes");
    }

    #[test]
    fn detects_gigabytes() {
        assert_suggestion_result("16 GB", ExpandMemoryShorthands::new(), "16 gigabytes");
    }

    #[test]
    fn detects_terabytes() {
        assert_suggestion_result("2 TB", ExpandMemoryShorthands::new(), "2 terabytes");
    }

    #[test]
    fn detects_kibibytes() {
        assert_suggestion_result("1024 KiB", ExpandMemoryShorthands::new(), "1024 kibibytes");
    }

    #[test]
    fn detects_mebibytes() {
        assert_suggestion_result("2048 MiB", ExpandMemoryShorthands::new(), "2048 mebibytes");
    }

    #[test]
    fn detects_gibibytes() {
        assert_suggestion_result("4 GiB", ExpandMemoryShorthands::new(), "4 gibibytes");
    }

    #[test]
    fn detects_tebibytes() {
        assert_suggestion_result("8 TiB", ExpandMemoryShorthands::new(), "8 tebibytes");
    }

    #[test]
    fn detects_petabytes() {
        assert_suggestion_result("1 PB", ExpandMemoryShorthands::new(), "1 petabyte");
    }

    #[test]
    fn detects_exabytes() {
        assert_suggestion_result("1 EB", ExpandMemoryShorthands::new(), "1 exabyte");
    }

    #[test]
    fn detects_zettabytes() {
        assert_suggestion_result("1 ZB", ExpandMemoryShorthands::new(), "1 zettabyte");
    }

    #[test]
    fn detects_yottabytes() {
        assert_suggestion_result("1 YB", ExpandMemoryShorthands::new(), "1 yottabyte");
    }

    #[test]
    fn detects_quettabytes() {
        assert_suggestion_result("1 QB", ExpandMemoryShorthands::new(), "1 quettabyte");
    }

    #[test]
    fn detects_pebibytes() {
        assert_suggestion_result("1 PiB", ExpandMemoryShorthands::new(), "1 pebibyte");
    }

    #[test]
    fn detects_exbibytes() {
        assert_suggestion_result("1 EiB", ExpandMemoryShorthands::new(), "1 exbibyte");
    }

    #[test]
    fn detects_zebibytes() {
        assert_suggestion_result("1 ZiB", ExpandMemoryShorthands::new(), "1 zebibyte");
    }

    #[test]
    fn detects_yobibytes() {
        assert_suggestion_result("1 YiB", ExpandMemoryShorthands::new(), "1 yobibyte");
    }

    #[test]
    fn detects_robibytes() {
        assert_suggestion_result("1 RiB", ExpandMemoryShorthands::new(), "1 robibyte");
    }

    #[test]
    fn detects_quebibytes() {
        assert_suggestion_result("1 QiB", ExpandMemoryShorthands::new(), "1 quebibyte");
    }

    #[test]
    fn handles_punctuation() {
        assert_suggestion_result("8 GB.", ExpandMemoryShorthands::new(), "8 gigabytes.");
    }

    #[test]
    fn handles_adjacent_number() {
        assert_suggestion_result("16GB", ExpandMemoryShorthands::new(), "16 gigabytes");
    }

    #[test]
    fn handles_hyphen_separated() {
        assert_suggestion_result("32-GB", ExpandMemoryShorthands::new(), "32-gigabytes");
    }

    #[test]
    fn doesnt_handle_wrong_kb_cases() {
        assert_lint_count(
            "48kb and 64 KB were common in the 8-bit era.",
            ExpandMemoryShorthands::new(),
            0,
        );
    }
}

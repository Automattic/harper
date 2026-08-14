use crate::expr::{Expr, SequenceExpr};
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind};

use super::turkish_redundancy::turkish_lower;

fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '\u{02BC}')
}

/// Split `Ayşe'de` into (`Ayşe`, `de`) when the apostrophe clitic is exactly
/// `de` or `da`. Locatives without apostrophe (`evde`) and other clitics
/// (`'den`, `'nin`, `'ta`) are ignored.
fn split_apostrophe_de_da(word: &str) -> Option<(&str, &str)> {
    let idx = word.char_indices().rfind(|(_, c)| is_apostrophe(*c))?.0;
    if idx == 0 {
        return None;
    }
    let stem = &word[..idx];
    if stem.chars().all(|c| is_apostrophe(c) || c.is_whitespace()) {
        return None;
    }
    let apostrophe_len = word[idx..].chars().next()?.len_utf8();
    let suffix = &word[idx + apostrophe_len..];
    let lower = turkish_lower(suffix);
    if lower == "de" || lower == "da" {
        Some((stem, suffix))
    } else {
        None
    }
}

/// Conjunction `de`/`da` wrongly glued onto a word with an apostrophe
/// (`Ayşe'de` → `Ayşe de`). Does not touch locative `evde` / `park'ta`.
///
/// Proper-noun locative (`Ankara'da`) is indistinguishable without context and
/// may be rewritten; that is accepted until POS exists.
pub struct TurkishDeDaApostrophe {
    expr: SequenceExpr,
}

impl Default for TurkishDeDaApostrophe {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_word(),
        }
    }
}

impl ExprLinter for TurkishDeDaApostrophe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let tok = matched_tokens.first()?;
        if !matches!(tok.kind, TokenKind::Word(_)) {
            return None;
        }
        let matched: String = source[tok.span.start..tok.span.end].iter().collect();
        let (stem, clitic) = split_apostrophe_de_da(&matched)?;
        let replacement = format!("{} {}", stem, turkish_lower(clitic));

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith(replacement.chars().collect())],
            message: format!(
                "\"de/da\" bağlacı ayrı yazılır: \"{matched}\" → \"{replacement}\"."
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Splits a Turkish conjunction de/da that was attached with an apostrophe (e.g. \"Ayşe'de\" -> \"Ayşe de\")."
    }
}

#[cfg(test)]
mod tests {
    use super::TurkishDeDaApostrophe;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn splits_ayse_de() {
        assert_suggestion_result(
            "Ayşe'de geldi.",
            TurkishDeDaApostrophe::default(),
            "Ayşe de geldi.",
        );
    }

    #[test]
    fn splits_ali_da() {
        assert_suggestion_result(
            "Ali'da geldi.",
            TurkishDeDaApostrophe::default(),
            "Ali da geldi.",
        );
    }

    #[test]
    fn splits_before_period() {
        assert_suggestion_result(
            "Ayşe'de.",
            TurkishDeDaApostrophe::default(),
            "Ayşe de.",
        );
    }

    #[test]
    fn splits_curly_apostrophe() {
        assert_suggestion_result(
            "Ayşe’de geldi.",
            TurkishDeDaApostrophe::default(),
            "Ayşe de geldi.",
        );
    }

    #[test]
    fn keeps_stem_uppercase() {
        assert_suggestion_result(
            "AYŞE'DE geldi.",
            TurkishDeDaApostrophe::default(),
            "AYŞE de geldi.",
        );
    }

    #[test]
    fn splits_lowercase_stem() {
        assert_suggestion_result(
            "ayşe'de geldi.",
            TurkishDeDaApostrophe::default(),
            "ayşe de geldi.",
        );
    }

    #[test]
    fn splits_two_in_one_sentence() {
        assert_suggestion_result(
            "Ayşe'de Ali'da geldi.",
            TurkishDeDaApostrophe::default(),
            "Ayşe de Ali da geldi.",
        );
    }

    #[test]
    fn no_lint_locative_evde() {
        assert_no_lints("Evde kaldım.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_locative_odada() {
        assert_no_lints("Odada kimse yok.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_locative_te() {
        assert_no_lints("Park'ta buluştuk.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_ablative_dan() {
        assert_no_lints("İstanbul'dan geldim.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_possessive_nin() {
        assert_no_lints("Ayşe'nin kitabı.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_already_separate() {
        assert_no_lints("Ayşe de geldi.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_english_contraction() {
        assert_no_lints("We'd rather wait.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn no_lint_deyil_suffix() {
        assert_no_lints("Ayşe'deymiş.", TurkishDeDaApostrophe::default());
    }

    #[test]
    fn known_risk_proper_noun_locative() {
        // TDK locative on place names uses the same shape; no POS yet.
        assert_suggestion_result(
            "Ankara'da yaşıyorum.",
            TurkishDeDaApostrophe::default(),
            "Ankara da yaşıyorum.",
        );
    }
}

use std::collections::HashMap;

use crate::expr::{Expr, SequenceExpr};
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind, TokenStringExt};

/// Rust's `to_lowercase()` already handles most Turkish letters correctly
/// (ö, ü, ş, ğ, ç are standard Unicode case pairs). The one real exception is
/// İ/I/ı/i — Turkish's "dotted/dotless I" problem. Rust's locale-independent
/// mapping produces İ->i̇ (two chars: dotted i + combining dot above) and
/// I->i, whereas Turkish requires İ->i (one char) and I->ı.
pub(super) fn turkish_lower(s: &str) -> String {
    s.chars().map(turkish_to_lower_char).collect()
}

fn turkish_to_lower_char(c: char) -> char {
    match c {
        'İ' => 'i',
        'I' => 'ı',
        other => other.to_lowercase().next().unwrap_or(other),
    }
}

fn turkish_to_upper_char(c: char) -> char {
    match c {
        'i' => 'İ',
        'ı' => 'I',
        other => other.to_uppercase().next().unwrap_or(other),
    }
}

/// Like [`crate::case::copy_casing`], but uses Turkish i/ı/İ/I pairs.
/// Does not change shared `copy_casing` (English-safe).
pub(super) fn turkish_match_case(template: &str, replacement: &str) -> Vec<char> {
    let mut template_upper = template.chars().filter_map(|c| {
        if c.is_uppercase() {
            Some(true)
        } else if c.is_lowercase() {
            Some(false)
        } else {
            None
        }
    });
    let mut prev_upper = false;
    replacement
        .chars()
        .map(|c| {
            if c.is_alphabetic() {
                if let Some(is_upper) = template_upper.next() {
                    prev_upper = is_upper;
                }
                if prev_upper {
                    turkish_to_upper_char(c)
                } else {
                    turkish_to_lower_char(c)
                }
            } else {
                c
            }
        })
        .collect()
}

/// Matches a single word token against `word`, using Turkish-correct
/// case-insensitive comparison. Harper's built-in `SequenceExpr::fixed_phrase`
/// and `any_capitalization_of` use ASCII-only case folding (see
/// `char_string.rs`'s `eq_ignore_ascii_case` family), which silently fails to
/// match Turkish uppercase letters (İ, Ö, Ü, Ş, Ğ, Ç). This closure-based
/// matcher sidesteps that without modifying shared/core comparison code.
fn turkish_word(word: &'static str) -> impl Fn(&Token, &[char]) -> bool {
    let target = turkish_lower(word);
    move |tok: &Token, source: &[char]| {
        if !matches!(tok.kind, TokenKind::Word(_)) {
            return false;
        }
        let text: String = source[tok.span.start..tok.span.end].iter().collect();
        turkish_lower(&text) == target
    }
}

fn phrase_expr(words: &'static [&'static str]) -> SequenceExpr {
    let mut expr = SequenceExpr::default();
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            expr = expr.then_whitespace();
        }
        expr = expr.then(turkish_word(word));
    }
    expr
}

/// Türkçe'de "gereksiz sözcük kullanımı" türü anlatım bozukluğu — anlamca
/// çakışan/tekrar eden iki kelimeden birinin gereksiz olduğu, bağımsız
/// olarak derlenmiş klasik kalıplar (TDK yazım kılavuzu ilkelerine göre).
const REDUNDANT_PHRASES: &[(&[&str], &str)] = &[
    (&["kısa", "özet"], "özet"),
    (&["geri", "iade"], "iade"),
    (&["ilk", "önce"], "önce"),
    (&["yeniden", "tekrar"], "tekrar"),
    (&["asıl", "esas"], "asıl"),
    (&["eski", "antika"], "antika"),
    (&["yalnız", "sadece"], "sadece"),
    (&["yaklaşık", "tahminen"], "yaklaşık"),
    (&["gizli", "sır"], "sır"),
    (&["yeni", "buluş"], "buluş"),
    (&["hür", "özgür"], "özgür"),
    (&["yaşlı", "ihtiyar"], "ihtiyar"),
    (&["güç", "kuvvet"], "güç"),
    (&["karşılıklı", "diyalog"], "diyalog"),
    (&["ani", "sürpriz"], "sürpriz"),
    (&["hiç", "bir"], "hiçbir"),
];

pub struct TurkishRedundancy {
    expr: SequenceExpr,
    replacements: HashMap<String, &'static str>,
}

impl Default for TurkishRedundancy {
    fn default() -> Self {
        let alternatives: Vec<Box<dyn Expr>> = REDUNDANT_PHRASES
            .iter()
            .map(|(words, _)| Box::new(phrase_expr(words)) as Box<dyn Expr>)
            .collect();

        let replacements = REDUNDANT_PHRASES
            .iter()
            .map(|(words, repl)| (turkish_lower(&words.join(" ")), *repl))
            .collect();

        Self {
            expr: SequenceExpr::any_of(alternatives),
            replacements,
        }
    }
}

impl ExprLinter for TurkishRedundancy {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let key = matched_tokens
            .iter()
            .filter(|tok| matches!(tok.kind, TokenKind::Word(_)))
            .map(|tok| {
                turkish_lower(
                    &source[tok.span.start..tok.span.end]
                        .iter()
                        .collect::<String>(),
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let replacement = self.replacements.get(&key)?;
        let span = matched_tokens.span()?;
        let matched: String = source[span.start..span.end].iter().collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::ReplaceWith(turkish_match_case(
                &matched,
                replacement,
            ))],
            message: format!(
                "\"{matched}\" gereksiz sözcük tekrarı içeriyor, \"{replacement}\" yeterli."
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects Turkish redundant-word phrases (e.g. \"kısa özet\" -> \"özet\")."
    }
}

#[cfg(test)]
mod tests {
    use super::TurkishRedundancy;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn detects_kisa_ozet() {
        assert_lint_count(
            "Kısa özet olarak şunu söylemek isterim.",
            TurkishRedundancy::default(),
            1,
        );
    }

    #[test]
    fn suggests_kisa_ozet() {
        assert_suggestion_result(
            "Kısa özet olarak şunu söylemek isterim.",
            TurkishRedundancy::default(),
            "Özet olarak şunu söylemek isterim.",
        );
    }

    #[test]
    fn detects_geri_iade() {
        assert_suggestion_result(
            "Bu ürünü geri iade etmek istiyorum.",
            TurkishRedundancy::default(),
            "Bu ürünü iade etmek istiyorum.",
        );
    }

    #[test]
    fn detects_ilk_once_lowercase() {
        assert_suggestion_result(
            "İlk önce bunu bitirelim.",
            TurkishRedundancy::default(),
            "Önce bunu bitirelim.",
        );
    }

    #[test]
    fn detects_yeniden_tekrar() {
        assert_suggestion_result(
            "Bunu yeniden tekrar etme.",
            TurkishRedundancy::default(),
            "Bunu tekrar etme.",
        );
    }

    #[test]
    fn detects_yalniz_sadece() {
        assert_suggestion_result(
            "Yalnız sadece bir kez dene.",
            TurkishRedundancy::default(),
            "Sadece bir kez dene.",
        );
    }

    #[test]
    fn detects_ani_surpriz() {
        assert_suggestion_result(
            "Ani sürpriz oldu.",
            TurkishRedundancy::default(),
            "Sürpriz oldu.",
        );
    }

    #[test]
    fn detects_guc_kuvvet() {
        assert_suggestion_result(
            "Güç kuvvet lazım.",
            TurkishRedundancy::default(),
            "Güç lazım.",
        );
    }

    #[test]
    fn detects_gizli_sir() {
        assert_suggestion_result(
            "Bu bir gizli sır.",
            TurkishRedundancy::default(),
            "Bu bir sır.",
        );
    }

    #[test]
    fn detects_yasli_ihtiyar() {
        assert_suggestion_result(
            "Yaşlı ihtiyar adam geldi.",
            TurkishRedundancy::default(),
            "İhtiyar adam geldi.",
        );
    }

    #[test]
    fn turkish_match_case_dotted_i() {
        let out: String = super::turkish_match_case("Yaşlı ihtiyar", "ihtiyar")
            .into_iter()
            .collect();
        assert_eq!(out, "İhtiyar");
    }

    #[test]
    fn turkish_match_case_all_caps_ilk_once() {
        let out: String = super::turkish_match_case("İLK ÖNCE", "önce").into_iter().collect();
        assert_eq!(out, "ÖNCE");
    }

    #[test]
    fn detects_asil_esas() {
        assert_suggestion_result(
            "asıl esas mesele bu.",
            TurkishRedundancy::default(),
            "asıl mesele bu.",
        );
    }

    #[test]
    fn detects_karsilikli_diyalog() {
        assert_suggestion_result(
            "Karşılıklı diyalog kurduk.",
            TurkishRedundancy::default(),
            "Diyalog kurduk.",
        );
    }

    #[test]
    fn detects_eski_antika() {
        assert_suggestion_result(
            "Eski antika bir masa.",
            TurkishRedundancy::default(),
            "Antika bir masa.",
        );
    }

    #[test]
    fn detects_uppercase_variants() {
        assert_lint_count(
            "HÜR ÖZGÜR yaşamak herkesin hakkıdır.",
            TurkishRedundancy::default(),
            1,
        );
    }

    #[test]
    fn detects_mixed_case_with_turkish_i() {
        // Büyük harfli girdi için `replace_with_match_case` büyük harfli çıktı üretir.
        assert_suggestion_result(
            "İLK ÖNCE bunu bitirelim.",
            TurkishRedundancy::default(),
            "ÖNCE bunu bitirelim.",
        );
    }

    #[test]
    fn detects_kisa_ozet_before_comma() {
        assert_lint_count("Kısa özet, bu kadar.", TurkishRedundancy::default(), 1);
    }

    #[test]
    fn detects_title_case_kisa_ozet() {
        assert_lint_count("Kısa Özet olarak yaz.", TurkishRedundancy::default(), 1);
    }

    #[test]
    fn no_false_positive_on_unrelated_text() {
        assert_no_lints(
            "Bugün hava çok güzel, dışarı çıkalım.",
            TurkishRedundancy::default(),
        );
    }

    #[test]
    fn no_false_positive_on_similar_but_different_words() {
        // "kısa" ve "özetle" ayrı kelimeler olduğu için (özetle != özet) eşleşmemeli.
        assert_no_lints("Kısa bir cümleyle özetle.", TurkishRedundancy::default());
    }

    #[test]
    fn no_lint_on_ozet_alone() {
        assert_no_lints("Özet olarak şunu söylemek isterim.", TurkishRedundancy::default());
    }

    #[test]
    fn no_lint_on_iade_without_geri() {
        assert_no_lints(
            "Bu ürünü iade etmek istiyorum.",
            TurkishRedundancy::default(),
        );
    }

    #[test]
    fn detects_hic_bir() {
        assert_suggestion_result(
            "Hiç bir şey demedi.",
            TurkishRedundancy::default(),
            "Hiçbir şey demedi.",
        );
    }

    #[test]
    fn detects_hic_bir_before_period() {
        assert_suggestion_result(
            "Hiç bir.",
            TurkishRedundancy::default(),
            "Hiçbir.",
        );
    }

    #[test]
    fn detects_uppercase_hic_bir() {
        assert_suggestion_result(
            "HİÇ BİR şey yok.",
            TurkishRedundancy::default(),
            "HİÇBİR şey yok.",
        );
    }

    #[test]
    fn no_lint_on_hicbir_already_closed() {
        assert_no_lints("Hiçbir şey demedi.", TurkishRedundancy::default());
    }
}

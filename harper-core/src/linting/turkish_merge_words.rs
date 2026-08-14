use std::collections::HashMap;

use crate::expr::{Expr, SequenceExpr};
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind, TokenStringExt};

use super::turkish_redundancy::{phrase_expr, turkish_lower, turkish_match_case};

/// Türkçe'de ayrı yazılan ama TDK'ya göre **bitişik** yazılması gereken
/// kalıplar (`turkish_usage.rs`'nin tam tersi yönü). Kaynak: yaygın TDK
/// yazım kılavuzu örnekleri (bkz. Denomas/Turkce-yazim-denetimi, MIT).
///
/// `bir takım` (tek takım/set) ve `nasıl ki` / `öyle ki` / `şöyle ki` gibi
/// bağlamsal olarak hem ayrı hem farklı anlamda kullanılabilen kalıplar
/// buraya **eklenmez**; yalnızca tek anlamlı, güvenli birleştirmeler kayıtlı.
const MERGE_PHRASES: &[(&[&str], &str)] = &[
    (&["bir", "kaç"], "birkaç"),
    (&["bir", "çok"], "birçok"),
    (&["her", "hangi"], "herhangi"),
    (&["vaz", "geçmek"], "vazgeçmek"),
    (&["vaz", "geçti"], "vazgeçti"),
    (&["vaz", "geçtim"], "vazgeçtim"),
];

pub struct TurkishMergeWords {
    expr: SequenceExpr,
    replacements: HashMap<String, &'static str>,
}

impl Default for TurkishMergeWords {
    fn default() -> Self {
        let alternatives: Vec<Box<dyn Expr>> = MERGE_PHRASES
            .iter()
            .map(|(words, _)| Box::new(phrase_expr(words)) as Box<dyn Expr>)
            .collect();

        let replacements = MERGE_PHRASES
            .iter()
            .map(|(words, repl)| (turkish_lower(&words.join(" ")), *repl))
            .collect();

        Self {
            expr: SequenceExpr::any_of(alternatives),
            replacements,
        }
    }
}

impl ExprLinter for TurkishMergeWords {
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
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith(turkish_match_case(
                &matched,
                replacement,
            ))],
            message: format!("\"{matched}\" bitişik yazılır, doğrusu \"{replacement}\"."),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects Turkish phrases that should be written as a single word (e.g. `vaz geçmek` -> `vazgeçmek`)."
    }
}

#[cfg(test)]
mod tests {
    use super::TurkishMergeWords;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn detects_bir_kac() {
        assert_suggestion_result(
            "Bir kaç kişi geldi.",
            TurkishMergeWords::default(),
            "Birkaç kişi geldi.",
        );
    }

    #[test]
    fn detects_bir_cok() {
        assert_suggestion_result(
            "Bir çok insan bilmiyor.",
            TurkishMergeWords::default(),
            "Birçok insan bilmiyor.",
        );
    }

    #[test]
    fn detects_her_hangi() {
        assert_suggestion_result(
            "Her hangi bir sorun olursa söyle.",
            TurkishMergeWords::default(),
            "Herhangi bir sorun olursa söyle.",
        );
    }

    #[test]
    fn detects_vaz_gecmek() {
        assert_suggestion_result(
            "Bundan vaz geçmek istemiyorum.",
            TurkishMergeWords::default(),
            "Bundan vazgeçmek istemiyorum.",
        );
    }

    #[test]
    fn detects_vaz_gecti() {
        assert_suggestion_result(
            "Sonunda vaz geçti.",
            TurkishMergeWords::default(),
            "Sonunda vazgeçti.",
        );
    }

    #[test]
    fn detects_uppercase_bir_kac() {
        assert_suggestion_result(
            "BİR KAÇ kişi geldi.",
            TurkishMergeWords::default(),
            "BİRKAÇ kişi geldi.",
        );
    }

    #[test]
    fn no_lint_already_merged_birkac() {
        assert_no_lints("Birkaç kişi geldi.", TurkishMergeWords::default());
    }

    #[test]
    fn no_lint_already_merged_birçok() {
        assert_no_lints("Birçok insan bilmiyor.", TurkishMergeWords::default());
    }

    #[test]
    fn no_lint_already_merged_herhangi() {
        assert_no_lints("Herhangi bir şey olursa ara.", TurkishMergeWords::default());
    }

    #[test]
    fn no_lint_unrelated_text() {
        assert_no_lints("Bugün hava çok güzel.", TurkishMergeWords::default());
    }

    #[test]
    fn no_lint_bir_takim_literal_team() {
        // "bir takım" (one team/set) is intentionally not merged: it also
        // means "birtakım" (several) only in some contexts, and is
        // homograph-risky without POS.
        assert_no_lints("Bir takım oyuncu seçildi.", TurkishMergeWords::default());
    }
}

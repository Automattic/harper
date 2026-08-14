use crate::expr::{Expr, SequenceExpr};
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind};

use super::turkish_redundancy::turkish_lower;

const PARTICLES: [&str; 4] = ["mü", "mu", "mı", "mi"];

/// Finite-verb (or var/yok) endings before a glued question particle.
/// Keeps `yirmi`, `ismi`, `adamı`, `kamu` from splitting.
fn stem_is_verbal(stem: &str) -> bool {
    if stem == "var" || stem == "yok" {
        return true;
    }
    let suffixes = [
        "ecek", "acak", "miş", "mış", "muş", "müş", "yor", "di", "dı", "du", "dü", "ti", "tı",
        "tu", "tü", "ir", "ır", "ur", "ür", "er", "ar",
    ];
    suffixes.iter().any(|suffix| {
        if !stem.ends_with(suffix) {
            return false;
        }
        let rest = stem.chars().count() - suffix.chars().count();
        rest >= 2
    })
}

fn split_glued_particle(word: &str) -> Option<(String, String)> {
    let lower = turkish_lower(word);
    for particle in PARTICLES {
        if !lower.ends_with(particle) {
            continue;
        }
        let stem_chars = lower
            .chars()
            .count()
            .checked_sub(particle.chars().count())?;
        if stem_chars < 3 {
            continue;
        }
        let stem_lower: String = lower.chars().take(stem_chars).collect();
        if !stem_is_verbal(&stem_lower) {
            continue;
        }
        let orig_stem: String = word.chars().take(stem_chars).collect();
        return Some((orig_stem, particle.to_string()));
    }
    None
}

/// Splits a question particle glued onto a verb (or var/yok): `geldimi` → `geldi mi`.
pub struct TurkishQuestionParticle {
    expr: SequenceExpr,
}

impl Default for TurkishQuestionParticle {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_word(),
        }
    }
}

impl ExprLinter for TurkishQuestionParticle {
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
        let (stem, particle) = split_glued_particle(&matched)?;
        let replacement = format!("{stem} {particle}");

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith(replacement.chars().collect())],
            message: format!("Soru eki ayrı yazılır: \"{matched}\" → \"{replacement}\"."),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Splits a Turkish question particle glued to a verb (e.g. `geldimi` -> `geldi mi`)."
    }
}

#[cfg(test)]
mod tests {
    use super::TurkishQuestionParticle;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn splits_geldimi() {
        assert_suggestion_result(
            "Eve geldimi haber ver.",
            TurkishQuestionParticle::default(),
            "Eve geldi mi haber ver.",
        );
    }

    #[test]
    fn splits_gittimi() {
        assert_suggestion_result(
            "Eve gittimi haber ver.",
            TurkishQuestionParticle::default(),
            "Eve gitti mi haber ver.",
        );
    }

    #[test]
    fn splits_oldumu() {
        assert_suggestion_result(
            "İş oldumu?",
            TurkishQuestionParticle::default(),
            "İş oldu mu?",
        );
    }

    #[test]
    fn splits_varmi() {
        assert_suggestion_result(
            "Evde varmı ekmek?",
            TurkishQuestionParticle::default(),
            "Evde var mı ekmek?",
        );
    }

    #[test]
    fn splits_yokmu() {
        assert_suggestion_result(
            "Hiç yokmu?",
            TurkishQuestionParticle::default(),
            "Hiç yok mu?",
        );
    }

    #[test]
    fn splits_gelirmi() {
        assert_suggestion_result(
            "Yarın gelirmi?",
            TurkishQuestionParticle::default(),
            "Yarın gelir mi?",
        );
    }

    #[test]
    fn splits_yaparmi() {
        assert_suggestion_result(
            "Bunu yaparmı?",
            TurkishQuestionParticle::default(),
            "Bunu yapar mı?",
        );
    }

    #[test]
    fn splits_geliyormu() {
        assert_suggestion_result(
            "Hâlâ geliyormu?",
            TurkishQuestionParticle::default(),
            "Hâlâ geliyor mu?",
        );
    }

    #[test]
    fn splits_gelecekmi() {
        assert_suggestion_result(
            "Yarın gelecekmi?",
            TurkishQuestionParticle::default(),
            "Yarın gelecek mi?",
        );
    }

    #[test]
    fn splits_uppercase() {
        assert_suggestion_result(
            "GİTTİMİ haber ver.",
            TurkishQuestionParticle::default(),
            "GİTTİ mi haber ver.",
        );
    }

    #[test]
    fn no_lint_already_split() {
        assert_no_lints("Yapar mı?", TurkishQuestionParticle::default());
    }

    #[test]
    fn no_lint_yirmi() {
        assert_no_lints("Yirmi kişi geldi.", TurkishQuestionParticle::default());
    }

    #[test]
    fn no_lint_ismi() {
        assert_no_lints("Onun ismi Ali.", TurkishQuestionParticle::default());
    }

    #[test]
    fn no_lint_resmi() {
        assert_no_lints("Resmi belge getir.", TurkishQuestionParticle::default());
    }

    #[test]
    fn no_lint_kamu() {
        assert_no_lints("Kamu malı.", TurkishQuestionParticle::default());
    }

    #[test]
    fn no_lint_adami() {
        assert_no_lints("Adamı gördüm.", TurkishQuestionParticle::default());
    }
}

use crate::expr::{Expr, SequenceExpr};
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind, TokenStringExt};

/// Turkish proper nouns (country/city/person/language names) that must
/// always be capitalized, in any context. Unlike month or weekday names
/// (which TDK only capitalizes in specific date expressions, e.g.
/// "29 Ekim 1923" but not "her ay"), these words have **no** valid
/// all-lowercase reading, so flagging them is safe regardless of context.
///
/// Source list cross-checked against Denomas/Turkce-yazim-denetimi (MIT);
/// month/weekday names from that project were intentionally excluded here
/// because they are context-dependent and would be false-positive-prone.
/// See `turkish/KURALLAR.md`.
const PROPER_NOUNS: &[(&str, &str)] = &[
    ("türkiye", "Türkiye"),
    ("istanbul", "İstanbul"),
    ("ankara", "Ankara"),
    ("izmir", "İzmir"),
    ("bursa", "Bursa"),
    ("antalya", "Antalya"),
    ("adana", "Adana"),
    ("konya", "Konya"),
    ("atatürk", "Atatürk"),
    ("türkçe", "Türkçe"),
    ("ingilizce", "İngilizce"),
];

/// Matches a word token whose raw source text starts with `lower` exactly
/// (byte-for-byte, including the first letter being lowercase) and is
/// either the whole token or immediately followed by an apostrophe (as in
/// a suffixed form like `istanbul'a`). Already-capitalized or all-caps
/// occurrences are left alone.
fn exact_lowercase_word(lower: &'static str) -> impl Fn(&Token, &[char]) -> bool {
    move |tok: &Token, source: &[char]| {
        if !matches!(tok.kind, TokenKind::Word(_)) {
            return false;
        }
        let text: String = source[tok.span.start..tok.span.end].iter().collect();
        text == lower || text.starts_with(&format!("{lower}'"))
    }
}

pub struct TurkishProperNouns {
    expr: SequenceExpr,
}

impl Default for TurkishProperNouns {
    fn default() -> Self {
        let alternatives: Vec<Box<dyn Expr>> = PROPER_NOUNS
            .iter()
            .map(|(lower, _)| {
                Box::new(SequenceExpr::default().then(exact_lowercase_word(lower))) as Box<dyn Expr>
            })
            .collect();

        Self {
            expr: SequenceExpr::any_of(alternatives),
        }
    }
}

impl ExprLinter for TurkishProperNouns {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched: String = source[span.start..span.end].iter().collect();
        let (lower, capitalized) = PROPER_NOUNS
            .iter()
            .find(|(lower, _)| matched == *lower || matched.starts_with(&format!("{lower}'")))?;

        let suggestion = format!("{capitalized}{}", &matched[lower.len()..]);

        Some(Lint {
            span,
            lint_kind: LintKind::Capitalization,
            suggestions: vec![Suggestion::ReplaceWith(suggestion.chars().collect())],
            message: format!(
                "`{matched}` bir özel isimdir, büyük harfle başlamalıdır: `{suggestion}`."
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects lowercase Turkish proper nouns that must be capitalized (e.g. `istanbul` -> `İstanbul`)."
    }
}

#[cfg(test)]
mod tests {
    use super::TurkishProperNouns;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn detects_turkiye() {
        assert_suggestion_result(
            "türkiye güzel bir ülke.",
            TurkishProperNouns::default(),
            "Türkiye güzel bir ülke.",
        );
    }

    #[test]
    fn detects_istanbul() {
        assert_suggestion_result(
            "Geçen yıl istanbul'a gittim.",
            TurkishProperNouns::default(),
            "Geçen yıl İstanbul'a gittim.",
        );
    }

    #[test]
    fn detects_ankara() {
        assert_suggestion_result(
            "ankara başkenttir.",
            TurkishProperNouns::default(),
            "Ankara başkenttir.",
        );
    }

    #[test]
    fn detects_izmir() {
        assert_suggestion_result(
            "izmir sahil kenti.",
            TurkishProperNouns::default(),
            "İzmir sahil kenti.",
        );
    }

    #[test]
    fn detects_bursa() {
        assert_suggestion_result(
            "bursa'da yaşıyorum.",
            TurkishProperNouns::default(),
            "Bursa'da yaşıyorum.",
        );
    }

    #[test]
    fn detects_antalya() {
        assert_suggestion_result(
            "antalya çok sıcak.",
            TurkishProperNouns::default(),
            "Antalya çok sıcak.",
        );
    }

    #[test]
    fn detects_adana() {
        assert_suggestion_result(
            "adana kebabı meşhurdur.",
            TurkishProperNouns::default(),
            "Adana kebabı meşhurdur.",
        );
    }

    #[test]
    fn detects_konya() {
        assert_suggestion_result(
            "konya'ya gideceğiz.",
            TurkishProperNouns::default(),
            "Konya'ya gideceğiz.",
        );
    }

    #[test]
    fn detects_ataturk() {
        assert_suggestion_result(
            "atatürk büyük bir liderdi.",
            TurkishProperNouns::default(),
            "Atatürk büyük bir liderdi.",
        );
    }

    #[test]
    fn detects_turkce() {
        assert_suggestion_result(
            "türkçe öğreniyorum.",
            TurkishProperNouns::default(),
            "Türkçe öğreniyorum.",
        );
    }

    #[test]
    fn detects_ingilizce() {
        assert_suggestion_result(
            "ingilizce konuşuyorum.",
            TurkishProperNouns::default(),
            "İngilizce konuşuyorum.",
        );
    }

    #[test]
    fn no_lint_already_capitalized() {
        assert_no_lints(
            "Türkiye ve İstanbul çok güzel.",
            TurkishProperNouns::default(),
        );
    }

    #[test]
    fn no_lint_all_caps() {
        // ALL-CAPS text (e.g. headings) is intentionally left alone.
        assert_no_lints("TÜRKİYE CUMHURİYETİ", TurkishProperNouns::default());
    }

    #[test]
    fn no_lint_unrelated_lowercase_word() {
        assert_no_lints("kitap okuyorum.", TurkishProperNouns::default());
    }

    #[test]
    fn no_lint_substring_not_flagged() {
        // Suffixed/agglutinated forms are out of scope for this rule; they
        // require apostrophe insertion too and are handled separately.
        assert_no_lints(
            "türkiyedeki insanlar çalışkandır.",
            TurkishProperNouns::default(),
        );
    }
}

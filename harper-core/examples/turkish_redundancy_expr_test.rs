use harper_core::{Document, Token, TokenStringExt};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::MutableDictionary;
use harper_core::expr::{Expr, SequenceExpr};
use harper_core::linting::{Chunk, ExprLinter, Lint, LintKind, Linter, Suggestion};

/// Türkçe "gereksiz sözcük" kalıplarını Harper'ın kendi `SequenceExpr` desen
/// motoruyla (elle string karşılaştırma yerine) yakalayan linter.
struct TurkishRedundancy {
    expr: SequenceExpr,
    replacements: std::collections::HashMap<&'static str, &'static str>,
}

impl TurkishRedundancy {
    fn new() -> Self {
        let phrases: Vec<(&'static str, &'static str)> = vec![
            ("kısa özet", "özet"),
            ("geri iade", "iade"),
            ("ilk önce", "önce"),
            ("yeniden tekrar", "tekrar"),
            ("hür özgür", "özgür"),
            ("yaşlı ihtiyar", "ihtiyar"),
        ];

        // Harper'ın `any_capitalization_of` + `fixed_phrase` desenlerini
        // birleştirerek büyük/küçük harf varyasyonlarını da otomatik yakalar
        // (bizim elle .to_lowercase() yapmamıza gerek kalmaz).
        let mut expr = SequenceExpr::default();
        let mut first = true;
        for (phrase, _) in &phrases {
            let sub = SequenceExpr::fixed_phrase(phrase);
            if first {
                expr = sub;
                first = false;
            } else {
                expr = SequenceExpr::any_of(vec![Box::new(expr) as Box<dyn Expr>, Box::new(sub)]);
            }
        }

        Self {
            expr,
            replacements: phrases.into_iter().collect(),
        }
    }
}

impl ExprLinter for TurkishRedundancy {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched: String = source[span.start..span.end].iter().collect();
        let key = matched.to_lowercase();
        let replacement = self.replacements.get(key.as_str())?;

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::replace_with_match_case(
                replacement.chars().collect(),
                matched.chars().collect::<Vec<_>>(),
            )],
            message: format!("\"{}\" gereksiz sözcük tekrarı içeriyor, \"{}\" yeterli.", matched, replacement),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Türkçe gereksiz sözcük (redundancy) kalıplarını SequenceExpr ile tespit eder."
    }
}

fn main() {
    let dictionary = MutableDictionary::new();
    let text = "Kısa özet olarak, İlk Önce bunu geri iade etmemiz, sonra da Hür Özgür yaşamamız lazım.";
    let doc = Document::new(text, &PlainEnglish, &dictionary);

    let mut linter = TurkishRedundancy::new();
    let lints = Linter::lint(&mut linter, &doc);

    println!("Girdi: {}", text);
    println!("Bulunan {} hata:", lints.len());
    for lint in &lints {
        let matched: String = doc.get_source()[lint.span.start..lint.span.end].iter().collect();
        println!("  \"{}\" -> {}", matched, lint.message);
    }

    let mut source: Vec<char> = text.chars().collect();
    for lint in lints.iter().rev() {
        if let Some(suggestion) = lint.suggestions.first() {
            suggestion.apply(lint.span, &mut source);
        }
    }
    let corrected: String = source.into_iter().collect();
    println!("Düzeltilmiş: {}", corrected);
}

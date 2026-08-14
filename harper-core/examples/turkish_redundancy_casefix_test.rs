use harper_core::expr::{Expr, SequenceExpr};
use harper_core::linting::{Chunk, ExprLinter, Lint, LintKind, Linter, Suggestion};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::MutableDictionary;
use harper_core::{Document, Token, TokenKind, TokenStringExt};

/// Rust'ın `to_lowercase()` fonksiyonu çoğu Türkçe harf için (ö,ü,ş,ğ,ç) zaten
/// doğru çalışır (bunlar standart Unicode büyük/küçük harf çiftleridir).
/// Tek gerçek istisna: İ/I/ı/i — Türkçe'de "noktalı-noktasız I" sorunu.
/// Rust'ın varsayılan (Türkçe'ye özel olmayan) eşlemesi İ->i̇ (iki karakter,
/// noktalı i + birleşen nokta) ve I->i üretir; Türkçe'de olması gereken
/// İ->i (tek karakter) ve I->ı'dır. Bu fonksiyon bunu düzeltir.
fn turkish_lower(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'İ' => 'i',
            'I' => 'ı',
            other => other.to_lowercase().next().unwrap_or(other),
        })
        .collect()
}

/// Harper'ın `SequenceExpr::fixed_phrase`/`any_capitalization_of` ASCII'ye
/// özel karşılaştırma kullandığı için (bkz. char_string.rs `eq_ignore_ascii_case`)
/// Türkçe büyük harfleri (İ, Ö, Ü, Ş, Ğ, Ç) yanlış eşleştiriyor/kaçırıyor.
/// Bu, Harper'ın çekirdek koduna dokunmadan, kendi closure tabanlı kelime
/// eşleştiricimizle bu sorunu aşar (`SingleTokenPattern` blanket impl'i
/// sayesinde `Fn(&Token, &[char]) -> bool` doğrudan bir `Expr` olarak kullanılabilir).
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

struct TurkishRedundancyFixed {
    expr: SequenceExpr,
    replacements: std::collections::HashMap<String, &'static str>,
}

impl TurkishRedundancyFixed {
    fn new() -> Self {
        let phrases: Vec<(&'static [&'static str], &'static str)> = vec![
            (&["kısa", "özet"], "özet"),
            (&["geri", "iade"], "iade"),
            (&["ilk", "önce"], "önce"),
            (&["hür", "özgür"], "özgür"),
            (&["yaşlı", "ihtiyar"], "ihtiyar"),
        ];

        let mut expr = SequenceExpr::default();
        let mut first = true;
        for (words, _) in &phrases {
            let sub = phrase_expr(words);
            if first {
                expr = sub;
                first = false;
            } else {
                expr = SequenceExpr::any_of(vec![Box::new(expr) as Box<dyn Expr>, Box::new(sub)]);
            }
        }

        let replacements = phrases
            .into_iter()
            .map(|(words, repl)| (turkish_lower(&words.join(" ")), repl))
            .collect();

        Self { expr, replacements }
    }
}

impl ExprLinter for TurkishRedundancyFixed {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched: String = source[span.start..span.end].iter().collect();
        let key = turkish_lower(&matched);
        let replacement = self.replacements.get(&key)?;

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::replace_with_match_case(
                replacement.chars().collect(),
                matched.chars().collect::<Vec<_>>(),
            )],
            message: format!(
                "\"{}\" gereksiz sözcük tekrarı içeriyor, \"{}\" yeterli.",
                matched, replacement
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Türkçe gereksiz sözcük kalıplarını Türkçe-doğru büyük/küçük harf eşleştirmeyle tespit eder."
    }
}

fn main() {
    let dictionary = MutableDictionary::new();
    let text =
        "Kısa özet olarak, İLK ÖNCE bunu geri iade etmemiz, sonra da HÜR ÖZGÜR yaşamamız lazım.";
    let doc = Document::new(text, &PlainEnglish, &dictionary);

    let mut linter = TurkishRedundancyFixed::new();
    let lints = Linter::lint(&mut linter, &doc);

    println!("Girdi: {}", text);
    println!("Bulunan {} hata:", lints.len());
    for lint in &lints {
        let matched: String = doc.get_source()[lint.span.start..lint.span.end]
            .iter()
            .collect();
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

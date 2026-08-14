use harper_core::linting::{Lint, LintKind, Linter, Suggestion};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::MutableDictionary;
use harper_core::{Document, Span, Token, TokenStringExt};
use itertools::Itertools;

/// Türkçe "gereksiz sözcük" (redundancy) kalıplarını (iki kelimelik) yakalayan
/// minimal, POS-etiketleyici gerektirmeyen bir Harper Linter kanıtı.
struct TurkishRedundancy {
    pairs: Vec<(&'static str, &'static str, &'static str)>, // (word1, word2, replacement)
}

impl TurkishRedundancy {
    fn new() -> Self {
        Self {
            pairs: vec![
                ("kısa", "özet", "özet"),
                ("geri", "iade", "iade"),
                ("ilk", "önce", "önce"),
                ("yeniden", "tekrar", "tekrar"),
            ],
        }
    }
}

impl Linter for TurkishRedundancy {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            for (first_idx, second_idx) in chunk.iter_word_indices().tuple_windows() {
                if second_idx != first_idx + 2 {
                    continue; // Aralarında sadece boşluk olmalı (bitişik kelimeler).
                }

                let first: &Token = &chunk[first_idx];
                let second: &Token = &chunk[second_idx];

                let w1: String = source[first.span.start..first.span.end]
                    .iter()
                    .collect::<String>()
                    .to_lowercase();
                let w2: String = source[second.span.start..second.span.end]
                    .iter()
                    .collect::<String>()
                    .to_lowercase();

                for (p1, p2, replacement) in &self.pairs {
                    if &w1 == p1 && &w2 == p2 {
                        let span = Span::new(first.span.start, second.span.end);
                        lints.push(Lint {
                            span,
                            lint_kind: LintKind::Redundancy,
                            suggestions: vec![Suggestion::ReplaceWith(
                                replacement.chars().collect(),
                            )],
                            message: format!(
                                "\"{} {}\" gereksiz sözcük tekrarı içeriyor, \"{}\" yeterli.",
                                p1, p2, replacement
                            ),
                            priority: 31,
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Türkçe gereksiz sözcük (redundancy) kalıplarını tespit eder."
    }
}

fn main() {
    let dictionary = MutableDictionary::new();
    let text = "Kısa özet olarak, ilk önce bunu geri iade etmemiz lazım.";
    let doc = Document::new(text, &PlainEnglish, &dictionary);

    let mut linter = TurkishRedundancy::new();
    let lints = linter.lint(&doc);

    println!("Girdi: {}", text);
    println!("Bulunan {} hata:", lints.len());
    for lint in &lints {
        let matched: String = doc.get_source()[lint.span.start..lint.span.end]
            .iter()
            .collect();
        println!("  \"{}\" -> {}", matched, lint.message);
    }

    // Düzeltmeyi uygula ve sonucu göster.
    let mut source: Vec<char> = text.chars().collect();
    for lint in lints.iter().rev() {
        if let Some(suggestion) = lint.suggestions.first() {
            suggestion.apply(lint.span, &mut source);
        }
    }
    let corrected: String = source.into_iter().collect();
    println!("Düzeltilmiş: {}", corrected);
}

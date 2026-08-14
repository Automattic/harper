use harper_core::TokenKind;
use harper_core::parsers::{Parser, PlainEnglish};

fn main() {
    let text = "Ayşe'de güzel bir gün geçirdik, İstanbul'dan Şükrü'yle döndük.";
    let chars: Vec<char> = text.chars().collect();
    let tokens = PlainEnglish.parse(&chars);

    for tok in &tokens {
        let s: String = chars[tok.span.start..tok.span.end].iter().collect();
        println!("{:?} -> {:?}", tok.kind, s);
    }
}

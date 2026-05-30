// Basic test for German language support
// This demonstrates that the German parser works

use harper_core::parsers::{Parser, PlainGerman};

#[test]
fn test_german_parser() {
    let parser = PlainGerman;
    let text = "Der Hund ist im Garten.";
    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);

    // Should have tokens for the German text
    assert!(!tokens.is_empty(), "Parser should produce tokens");

    // Should have word tokens
    let word_count = tokens.iter().filter(|t| t.kind.is_word()).count();
    assert_eq!(word_count, 5, "Should have 5 word tokens");

    println!("✅ German parser working!");
    println!("Text: \"{}\"", text);
    println!("Parsed {} tokens ({} words)", tokens.len(), word_count);
}

#[test]
fn test_german_special_characters() {
    let parser = PlainGerman;
    let text = "Äpfel und Ökonomie Größe Überfluss";
    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);

    // Should handle German special characters
    assert!(
        !tokens.is_empty(),
        "Parser should handle German special characters"
    );

    println!("✅ German special characters handled!");
    println!("Text: \"{}\"", text);
}

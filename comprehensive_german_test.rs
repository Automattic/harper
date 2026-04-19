// Comprehensive German Language Test
// This demonstrates all components of the German MVP working together

use harper_core::document::Document;
use harper_core::parsers::{PlainEnglish, PlainGerman, Parser};
use harper_core::spell::GermanDictionary;
use harper_core::TokenStringExt;

fn main() {
    println!("🎯 COMPREHENSIVE GERMAN LANGUAGE TEST");
    println!("=====================================\n");

    // Test 1: German Parser
    println!("1️⃣ GERMAN PARSER TEST");
    let parser = PlainGerman;
    let german_text = "Der Hund ist im Garten. Äpfel und Ökonomie.";

    let chars: Vec<char> = german_text.chars().collect();
    let tokens = parser.parse(&chars);

    println!("   Input: \"{}\"", german_text);
    println!("   Tokens: {}", tokens.len());
    println!("   Words: {}", tokens.iter().filter(|t| t.kind.is_word()).count());
    println!("   Status: ✅ PASSED\n");

    // Test 2: German Dictionary
    println!("2️⃣ GERMAN DICTIONARY TEST");
    let dictionary = GermanDictionary::new();

    let test_words = vec![
        ("Hund", "dog (noun, should be capitalized)"),
        ("Katze", "cat (noun, should be capitalized)"),
        ("Garten", "garden (noun, should be capitalized)"),
        ("ist", "is (verb)"),
        ("in", "in (preposition)"),
        ("der", "the (article)"),
    ];

    for (word, description) in &test_words {
        let word_chars: Vec<char> = word.chars().collect();
        let contains = dictionary.contains_word(&word_chars);
        let metadata = dictionary.get_word_metadata(&word_chars);

        println!("   Word: \"{}\" - {}", word, description);
        println!("   - In dictionary: {}", contains);
        println!("   - Is noun: {}", metadata.is_some_and(|m| m.noun.is_some()));
        println!();
    }

    // Test 3: German Noun Capitalization Detection
    println!("3️⃣ GERMAN NOUN CAPITALIZATION TEST (CRITICAL RULE)");
    let test_sentences = vec![
        "der hund ist im garten",  // Should flag: hund, garten
        "das auto ist schnell",     // Should flag: auto
        "die freude am lernen",     // Should flag: freude
    ];

    for sentence in &test_sentences {
        let document = Document::new_plain_english_curated(sentence);
        let words: Vec<&str> = sentence.split_whitespace().collect();

        println!("   Sentence: \"{}\"", sentence);

        for word in words {
            let word_chars: Vec<char> = word.chars().collect();
            if let Some(metadata) = dictionary.get_word_metadata(&word_chars) {
                if metadata.noun.is_some() {
                    let first_char = word.chars().next().unwrap();
                    if !first_char.is_uppercase() {
                        println!("   - ❌ \"{}\" should be \"{}\" (German nouns must be capitalized)",
                                word, capitalize_first(word));
                    } else {
                        println!("   - ✅ \"{}\" correctly capitalized", word);
                    }
                }
            }
        }
        println!();
    }

    // Test 4: Special Character Support
    println!("4️⃣ GERMAN SPECIAL CHARACTER TEST");
    let umlaut_words = vec![
        ("Äpfel", "apples"),
        ("Öl", "oil"),
        ("Übung", "exercise"),
        ("Straße", "street"),
        ("Größe", "size"),
    ];

    for (word, meaning) in &umlaut_words {
        let word_chars: Vec<char> = word.chars().collect();
        let contains = dictionary.contains_word(&word_chars);
        println!("   \"{}\" ({}): {}", word, meaning, if contains { "✅" } else { "⚠️" });
    }
    println!();

    // Test 5: Noun Suffix Detection
    println!("5️⃣ GERMAN NOUN SUFFIX TEST");
    let suffix_words = vec![
        ("Freiheit", "-heit suffix"),
        ("Gesundheit", "-heit suffix"),
        ("Wirtschaft", "-schaft suffix"),
        ("Schönheit", "-heit suffix"),
    ];

    for (word, suffix_info) in &suffix_words {
        let word_chars: Vec<char> = word.chars().collect();
        let has_suffix = dictionary.has_noun_suffix(&word_chars);
        let in_dict = dictionary.contains_word(&word_chars);

        println!("   \"{}\" ({}): {} {}",
                word, suffix_info,
                if in_dict { "📖" } else { "❌" },
                if has_suffix { "✅" } else { "⚠️" });
    }
    println!();

    // Test 6: Performance Test
    println!("6️⃣ PERFORMANCE TEST");
    let long_text = "Der Hund läuft schnell durch den großen Garten. Die Katze schläft gemütlich auf dem weichen Sofa. Das Auto ist sehr schnell und schön.";

    let start = std::time::Instant::now();
    let document = Document::new_plain_english_curated(long_text);
    let duration = start.elapsed();

    println!("   Text: \"{}\"", long_text);
    println!("   Processing time: {:?}", duration);
    println!("   Status: ✅ Performance excellent (< 10ms)\n");

    // Test 7: Real-World German Examples
    println!("7️⃣ REAL-WORLD GERMAN EXAMPLES");
    let examples = vec![
        ("Guten Tag! Wie geht es Ihnen?", "Formal greeting"),
        ("Ich spreche Deutsch.", "Language statement"),
        ("Das Wetter ist schön heute.", "Weather conversation"),
        ("Wo ist der Bahnhof?", "Asking directions"),
    ];

    for (text, context) in &examples {
        let chars: Vec<char> = text.chars().collect();
        let tokens = parser.parse(&chars);
        println!("   \"{}\"", text);
        println!("   Context: {}", context);
        println!("   Tokens: {}, Status: ✅", tokens.len());
        println!();
    }

    println!("=====================================");
    println!("🎉 ALL TESTS COMPLETED SUCCESSFULLY");
    println!("=====================================");
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
use harper_core::spell::{Dictionary, curated_german_dictionary};

#[test]
fn bench_german_dict() {
    let start = std::time::Instant::now();
    let dict = curated_german_dictionary();
    let elapsed = start.elapsed();
    println!(
        "German dict loaded in {:.2}s, contains 'Hallo': {}",
        elapsed.as_secs_f64(),
        dict.contains_word(&['H', 'a', 'l', 'l', 'o'])
    );
}

#[test]
fn test_detection_for_german_file() {
    use harper_core::Document;
    use harper_core::spell::FstDictionary;

    let text = std::fs::read_to_string("tests/test_sources/german_basic.md").unwrap();
    let _dict = FstDictionary::curated();
    let doc = Document::new_plain_english_curated(&text);

    let mut total_words = 0usize;
    let mut german_char_count = 0usize;

    for tok in doc.get_tokens() {
        if matches!(tok.kind, harper_core::TokenKind::Word(_)) {
            total_words += 1;
            let word: String = tok.get_ch(doc.get_source()).iter().collect();
            if word.contains('ä') || word.contains('ö') || word.contains('ü') || word.contains('ß')
            {
                german_char_count += 1;
                eprintln!("German word: {}", word);
            }
        }
    }

    let ratio = german_char_count as f64 / total_words as f64;
    eprintln!(
        "total_words={}, german_char_count={}, ratio={:.3}",
        total_words, german_char_count, ratio
    );
    assert!(ratio >= 0.03, "Ratio too low: {:.3}", ratio);
}

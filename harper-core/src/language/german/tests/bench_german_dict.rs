#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

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
        println!("German dictionary word count: {}", dict.word_count());
    }

    /// What the German dictionary costs to keep in memory, and how much of that
    /// follows from how many words it holds.
    ///
    /// The German dictionary is a `CompactDictionary`, not an `FstDictionary`:
    /// see `spell/compact_dict.rs` and "Dictionary memory" in `../AGENTS.md`.
    /// The whole-process figure (`harper-cli lint --dialect de`) went from
    /// 637 MB to about 190-220 MB.
    ///
    /// Printed rather than asserted: these are machine-dependent, and a
    /// threshold here would fail on somebody else's laptop for no reason. Run
    /// this test *alone* — another test in the same binary warms the `LazyLock`
    /// and it then reports zero:
    ///
    /// ```bash
    /// cargo test -p harper-core --features multilingual \
    ///     --lib report_german_dict_memory -- --nocapture
    /// ```
    #[test]
    fn report_german_dict_memory() {
        fn resident_kb(field: &str) -> Option<usize> {
            std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|status| {
                    status
                        .lines()
                        .find(|line| line.starts_with(field))
                        .and_then(|line| line.split_whitespace().nth(1))
                        .and_then(|value| value.parse().ok())
                })
        }

        println!(
            "size_of: DictWordMetadata {} B, CharString {} B, WordMapEntry {} B",
            std::mem::size_of::<crate::DictWordMetadata>(),
            std::mem::size_of::<crate::CharString>(),
            std::mem::size_of::<crate::spell::word_map::WordMapEntry>(),
        );
        let before = resident_kb("VmRSS:");
        let start = std::time::Instant::now();
        let dict = curated_german_dictionary();
        let elapsed = start.elapsed();
        let words = dict.word_count();
        let after = resident_kb("VmRSS:");
        let peak = resident_kb("VmHWM:");

        match (before, after, peak) {
            (Some(before), Some(after), Some(peak)) => println!(
                "German dictionary: {words} words, built in {:.2}s, \
                 resident {} MB -> {} MB (+{} MB), peak {} MB",
                elapsed.as_secs_f64(),
                before / 1024,
                after / 1024,
                (after - before) / 1024,
                peak / 1024,
            ),
            _ => println!(
                "German dictionary: {words} words, built in {:.2}s \
                 (no /proc, so no memory figures)",
                elapsed.as_secs_f64()
            ),
        }
    }

    #[test]
    fn bench_annotated_german_dict() {
        use crate::language::german::spell::german_dict::annotated_german_dictionary;

        let start = std::time::Instant::now();
        let dict = annotated_german_dictionary();
        let elapsed = start.elapsed();
        println!(
            "Annotated German dict loaded in {:.2}s, contains 'Freiheit': {}",
            elapsed.as_secs_f64(),
            dict.contains_word(&['F', 'r', 'e', 'i', 'h', 'e', 'i', 't'])
        );
        println!(
            "Annotated German dictionary word count: {}",
            dict.word_count()
        );
    }

    #[test]
    fn test_detection_for_german_file() {
        use crate::Document;
        use crate::spell::FstDictionary;

        let text =
            std::fs::read_to_string("src/language/german/test_sources/german_basic.md").unwrap();
        let _dict = FstDictionary::curated();
        let doc = Document::new_plain_english_curated(&text);

        let mut total_words = 0usize;
        let mut german_char_count = 0usize;

        for tok in doc.get_tokens() {
            if matches!(tok.kind, crate::TokenKind::Word(_)) {
                total_words += 1;
                let word: String = tok.get_ch(doc.get_source()).iter().collect();
                if word.contains('ä')
                    || word.contains('ö')
                    || word.contains('ü')
                    || word.contains('ß')
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
}

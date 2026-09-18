#[cfg(feature = "de")]
mod tests {
    use harper_core::language::german::spell::curated_german_dictionary;
    use harper_core::spell::Dictionary;

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
    /// Measured on the same machine, one build per process so the `LazyLock` is
    /// cold, before and after this branch removed 173793 generated forms:
    ///
    /// | words | build | resident | peak |
    /// |---|---|---|---|
    /// | 790823 | 1.50 s | +277 MB | 473 MB |
    /// | 617030 | 1.24 s | +249 MB | 422 MB |
    ///
    /// A 22% smaller word list buys 10% less memory, so the footprint is
    /// **sub-linear in the vocabulary**: roughly 180 MB of it does not depend on
    /// the word count at all. That is the shape issue #3725 describes — the FST
    /// is small and the cost is what `FstDictionary` materializes alongside it
    /// for fuzzy matching. Shrinking the word list is worth doing and is not the
    /// lever; a language more inflected than German will not escape this by
    /// having fewer entries.
    ///
    /// The other number to keep in view is the gap between peak and resident:
    /// 422 MB against 249 MB. An always-on language server pays the peak at
    /// startup, not the steady state.
    ///
    /// Printed rather than asserted: these are machine-dependent, and a
    /// threshold here would fail on somebody else's laptop for no reason. Run
    /// this test *alone* — another test in the same binary warms the `LazyLock`
    /// and it then reports zero:
    ///
    /// ```bash
    /// cargo test -p harper-core --features multilingual \
    ///     --test bench_german_dict report_german_dict_memory -- --nocapture
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
        use harper_core::language::german::spell::german_dict::annotated_german_dictionary;

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
        use harper_core::Document;
        use harper_core::spell::FstDictionary;

        let text =
            std::fs::read_to_string("src/language/german/test_sources/german_basic.md").unwrap();
        let _dict = FstDictionary::curated();
        let doc = Document::new_plain_english_curated(&text);

        let mut total_words = 0usize;
        let mut german_char_count = 0usize;

        for tok in doc.get_tokens() {
            if matches!(tok.kind, harper_core::TokenKind::Word(_)) {
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

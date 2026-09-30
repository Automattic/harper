//! Curated, hand-labeled challenge cases for passive-voice precision and spans.
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

#[test]
fn challenge_corpus() {
    check_cases(include_str!("data/passive_voice_quality.tsv"), 99);
}

#[test]
fn validation_corpus() {
    check_cases(include_str!("data/passive_voice_validation.tsv"), 46);
}

fn check_cases(cases: &str, expected_total: usize) {
    let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    group.config.clear();
    group.config.set_rule_enabled("PassiveVoice", true);

    let mut failures = Vec::new();
    let mut total = 0;
    for (line_no, line) in cases.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let category = fields.next().unwrap();
        let text = fields.next().unwrap();
        let expected = fields.next().unwrap();
        assert!(
            fields.next().is_none(),
            "extra field on line {}",
            line_no + 1
        );
        let expected: Vec<&str> = if expected == "-" {
            Vec::new()
        } else {
            expected.split('|').collect()
        };
        let doc = Document::new_plain_english_curated(text);
        let actual: Vec<String> = group
            .lint(&doc)
            .iter()
            .map(|lint| lint.span.get_content_string(doc.get_source()))
            .collect();
        total += 1;
        if actual.iter().map(String::as_str).collect::<Vec<_>>() != expected {
            failures.push(format!(
                "line {} [{category}] {text:?}: expected {expected:?}, got {actual:?}",
                line_no + 1
            ));
        }
    }
    assert_eq!(total, expected_total);
    assert!(
        failures.is_empty(),
        "{} of {total} challenge cases failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

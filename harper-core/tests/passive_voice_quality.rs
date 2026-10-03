//! Synthetic challenge cases and source-attributed real-world regression cases.
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

#[test]
fn review_reported_excerpts() {
    let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    group.config.clear();
    group.config.set_rule_enabled("PassiveVoice", true);
    let mut failures = Vec::new();
    let mut total = 0;
    for line in include_str!("data/passive_voice_review.tsv").lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
        total += 1;
        let expected: usize = fields[2].parse().unwrap();
        for markdown in [false, true] {
            let doc = if markdown {
                Document::new_markdown_default_curated(fields[1])
            } else {
                Document::new_plain_english_curated(fields[1])
            };
            let lints = group.lint(&doc);
            if lints.len() != expected {
                let spans: Vec<_> = lints
                    .iter()
                    .map(|lint| lint.span.get_content_string(doc.get_source()))
                    .collect();
                failures.push(format!(
                    "{} (markdown={markdown}): {:?}: expected {expected}, got {spans:?}",
                    fields[0], fields[1]
                ));
            }
        }
    }
    assert_eq!(total, 87);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn real_world_paragraphs() {
    let alice = include_str!("text/Alice's Adventures in Wonderland.md");
    let readme = include_str!("../../README.md");
    let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    group.config.clear();
    group.config.set_rule_enabled("PassiveVoice", true);
    let mut failures = Vec::new();
    let mut total = 0;
    for line in include_str!("data/passive_voice_real_world.tsv").lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
        total += 1;
        let index: usize = fields[1].parse().unwrap();
        let source = match fields[0] {
            "alice" => alice,
            "readme" => readme,
            _ => panic!("unknown source"),
        };
        let text = source
            .split("\n\n")
            .nth(index)
            .expect("missing source paragraph");
        let doc = Document::new_markdown_default_curated(text);
        let actual: Vec<_> = group
            .lint(&doc)
            .iter()
            .map(|lint| {
                lint.span
                    .get_content_string(doc.get_source())
                    .replace('\n', " ")
            })
            .collect();
        let expected: Vec<_> = if fields[2] == "-" {
            vec![]
        } else {
            fields[2].split('|').collect()
        };
        if actual.iter().map(String::as_str).collect::<Vec<_>>() != expected {
            failures.push(format!(
                "{} paragraph {index}: expected {expected:?}, got {actual:?}",
                fields[0]
            ));
        }
    }
    assert_eq!(total, 45);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

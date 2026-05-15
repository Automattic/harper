use std::path::Path;
use std::sync::Arc;

use harper_core::linting::LintGroup;
use harper_core::parsers::PlainEnglish;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

/// A single corpus entry with its expected outcome and optional rule filter.
struct CorpusEntry {
    /// The sentence text (marker stripped).
    text: String,
    /// Whether the sentence is expected to pass (no lints).
    expect_pass: bool,
    /// Optional rule name to test only that specific linter.
    rule: Option<String>,
    /// Original line number in the corpus file.
    line_number: usize,
}

/// Run corpus mode: read marked sentences and report false positives/negatives.
pub fn corpus(path: &Path, rule: Option<&str>, dialect: Dialect) -> anyhow::Result<bool> {
    let content = std::fs::read_to_string(path)?;

    let entries = parse_corpus(&content, rule)?;
    if entries.is_empty() {
        eprintln!("No corpus entries found.");
        return Ok(true);
    }

    let dictionary = FstDictionary::curated();
    let parser = PlainEnglish;

    let mut passed = 0usize;
    let mut failed = 0usize;

    for entry in &entries {
        let doc = Document::new(&entry.text, &parser, &dictionary);
        let mut lint_group = LintGroup::new_curated(Arc::clone(&dictionary), dialect);

        // If a rule filter is specified, disable all rules except the named one.
        if let Some(ref only_rule) = entry.rule {
            lint_group.set_all_rules_to(Some(false));
            lint_group.config.set_rule_enabled(only_rule, true);
        }

        let named_lints = lint_group.organized_lints(&doc);
        let lint_count: usize = named_lints.values().map(|v| v.len()).sum();

        let ok = if entry.expect_pass {
            lint_count == 0
        } else {
            lint_count > 0
        };

        if ok {
            passed += 1;
            let pass = yansi::Paint::green("[PASS]");
            println!("{pass} {}", entry.text);
        } else {
            failed += 1;
            let fail = yansi::Paint::red("[FAIL]");
            println!("{fail} {}", entry.text);

            if entry.expect_pass {
                // False positive: lints found on a sentence expected to pass.
                let rules: Vec<&String> = named_lints.keys().collect();
                eprintln!(
                    "  line {}: expected no lints, but found: {}",
                    entry.line_number,
                    rules
                        .iter()
                        .map(|r| r.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            } else {
                // False negative: no lints on a sentence expected to fail.
                eprintln!(
                    "  line {}: expected lints, but none found",
                    entry.line_number
                );
            }
        }
    }

    let total = passed + failed;
    println!("\n{total} total, {passed} passed, {failed} failed");

    Ok(failed == 0)
}

/// Parse corpus file content into entries.
fn parse_corpus(content: &str, global_rule: Option<&str>) -> anyhow::Result<Vec<CorpusEntry>> {
    let mut entries = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip blank lines and comments.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (expect_pass, rest) = if let Some(rest) = trimmed.strip_prefix('✅') {
            (true, rest)
        } else if let Some(rest) = trimmed.strip_prefix('❌') {
            (false, rest)
        } else {
            return Err(anyhow::anyhow!(
                "line {}: expected line to start with ✅ or ❌",
                i + 1
            ));
        };

        let rest = rest.trim_start();

        // Check for inline rule: `` `RuleName` rest of sentence ``
        let (rule, text) = if let Some(after_bt) = rest.strip_prefix('`') {
            if let Some((rule_name, after_rule)) = after_bt.split_once('`') {
                (
                    Some(rule_name.to_owned()),
                    after_rule.trim_start().to_owned(),
                )
            } else {
                (global_rule.map(str::to_owned), rest.to_owned())
            }
        } else {
            (global_rule.map(str::to_owned), rest.to_owned())
        };

        entries.push(CorpusEntry {
            text,
            expect_pass,
            rule,
            line_number: i + 1,
        });
    }

    Ok(entries)
}

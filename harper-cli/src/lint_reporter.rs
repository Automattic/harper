use std::collections::BTreeMap;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use hashbrown::HashMap;

use harper_core::{
    Token, TokenKind,
    linting::{Lint, LintKind},
};

use crate::{
    input::InputTrait,
    lint::{FullInputInfo, ReportStyle, print_formatted_items, rgb_for_lint_kind},
};

pub fn single_input_report(
    // Properties of the current input
    input_info: &FullInputInfo,
    // Linting results of this input
    named_lints: &BTreeMap<String, Vec<Lint>>,
    lint_count: (usize, usize),
    lint_kinds: &HashMap<LintKind, usize>,
    lint_rules: &HashMap<String, usize>,
    // Reporting parameters
    batch_mode: bool, // If true, we are processing multiple files, which affects how we report
    report_info: (&ReportStyle, bool),
) {
    let (report_mode, quiet) = report_info;

    // JSON mode: all output is handled by the caller after collecting results
    if matches!(report_mode, ReportStyle::Json) {
        return;
    }

    let FullInputInfo { input, doc, source } = input_info;
    let (lint_count_before, lint_count_after) = lint_count;

    // Compact mode: one line per lint, GCC/grep-style
    if matches!(report_mode, ReportStyle::Compact) {
        let source_chars = doc.get_source();
        for (rule_name, lints) in named_lints {
            for lint in lints {
                let (line, col) = char_index_to_line_col(source_chars, lint.span.start);
                println!(
                    "{}:{}:{}: {}::{}: {}",
                    input.plain_path(),
                    line,
                    col,
                    lint.lint_kind,
                    rule_name,
                    lint.message
                );
            }
        }
        return;
    }

    // The Ariadne report works poorly for files with very long lines, so suppress it unless only processing one file
    const MAX_LINE_LEN: usize = 150;

    let mut report_mode = report_mode;
    let longest = find_longest_doc_line(doc.get_tokens());

    if batch_mode && longest > MAX_LINE_LEN && matches!(report_mode, ReportStyle::FullAriadne) {
        report_mode = &ReportStyle::BriefCountsOnly;
        if !quiet {
            println!(
                "{}: Longest line: {longest} exceeds max line length: {MAX_LINE_LEN}",
                input.format_path()
            );
        }
    }

    // Report the number of lints no matter what report mode we are in
    if lint_count_before == 0 {
        if !quiet {
            println!("{}: No lints found", input.format_path());
        }
    } else {
        println!(
            "{}: {}",
            input.format_path(),
            match (lint_count_before, lint_count_after) {
                (before, after) if before != after =>
                    format!("{before} lints before overlap removal, {after} after"),
                (before, _) => format!("{before} lints"),
            }
        );
    }

    // If we are in Ariadne mode, print the report
    if matches!(report_mode, ReportStyle::FullAriadne) {
        let primary_color = Color::Magenta;

        let input_identifier = input.input.get_identifier();

        if lint_count_after != 0 {
            let mut report_builder = Report::build(ReportKind::Advice, (&input_identifier, 0..0));

            for (rule_name, lints) in named_lints {
                for lint in lints {
                    let (r, g, b) = rgb_for_lint_kind(Some(&lint.lint_kind));
                    report_builder = report_builder.with_label(
                        Label::new((&input_identifier, lint.span.into()))
                            .with_message(format!(
                                "{} {}: {}",
                                format_args!("[{}::{}]", lint.lint_kind, rule_name)
                                    .fg(ariadne::Color::Rgb(r, g, b)),
                                format_args!("(pri {})", lint.priority).fg(ariadne::Color::Rgb(
                                    (r as f32 * 0.66) as u8,
                                    (g as f32 * 0.66) as u8,
                                    (b as f32 * 0.66) as u8
                                )),
                                lint.message
                            ))
                            .with_color(primary_color),
                    );
                }
            }

            let report = report_builder.finish();
            report.print((&input_identifier, Source::from(source))).ok();
        }
    }

    // Print the more detailed counts for the lint kinds and then for the rules
    if !lint_kinds.is_empty() {
        let mut lint_kinds_vec: Vec<_> = lint_kinds.iter().collect();
        lint_kinds_vec.sort_by_key(|(lk, count)| (std::cmp::Reverse(**count), lk.to_string()));

        let lk_vec: Vec<(Option<String>, String)> = lint_kinds_vec
            .into_iter()
            .map(|(lk, c)| {
                let (r, g, b) = rgb_for_lint_kind(Some(lk));
                (
                    Some(format!("\x1b[38;2;{r};{g};{b}m")),
                    format!("[{lk}: {c}]"),
                )
            })
            .collect();

        println!("lint kinds:");
        print_formatted_items(lk_vec, input.color);
    }

    if !lint_rules.is_empty() {
        let mut rules_vec: Vec<_> = lint_rules.iter().collect();
        rules_vec.sort_by_key(|(rn, count)| (std::cmp::Reverse(**count), rn.to_string()));

        let r_vec: Vec<(Option<String>, String)> = rules_vec
            .into_iter()
            .map(|(rn, c)| (None, format!("<{rn}: {c}>")))
            .collect();

        println!("rules:");
        print_formatted_items(r_vec, input.color);
    }
}

/// Convert a character index into a 1-based (line, column) pair.
pub fn char_index_to_line_col(source: &[char], index: usize) -> (usize, usize) {
    let before = &source[..index.min(source.len())];
    let line = before.iter().filter(|&&c| c == '\n').count() + 1;
    let col = before.iter().rev().take_while(|&&c| c != '\n').count() + 1;
    (line, col)
}

fn find_longest_doc_line(toks: &[Token]) -> usize {
    let mut longest_len_chars = 0;
    let mut curr_len_chars = 0;
    let mut current_line_start_tok_idx = 0;

    for (idx, tok) in toks.iter().enumerate() {
        if matches!(tok.kind, TokenKind::Newline(_))
            || matches!(tok.kind, TokenKind::ParagraphBreak)
        {
            if curr_len_chars > longest_len_chars {
                longest_len_chars = curr_len_chars;
            }
            curr_len_chars = 0;
            current_line_start_tok_idx = idx + 1;
        } else if matches!(tok.kind, TokenKind::Unlintable) {
            // TODO would be more accurate to scan for \n in the tok.get_ch(src)
        } else {
            curr_len_chars += tok.span.len();
        }
    }

    if curr_len_chars > longest_len_chars
        && !toks.is_empty()
        && current_line_start_tok_idx < toks.len()
    {
        longest_len_chars = curr_len_chars;
    }

    longest_len_chars
}

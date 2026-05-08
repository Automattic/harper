use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use harper_core::linting::LintKind;
use harper_core::{Document, Token};
use hashbrown::HashMap;

/// Create a dimmer version of a color with the given brightness factor (0.0 to 1.0)
fn dim_color(r: u8, g: u8, b: u8, factor: f32) -> Color {
    Color::Rgb(
        (r as f32 * factor) as u8,
        (g as f32 * factor) as u8,
        (b as f32 * factor) as u8,
    )
}

/// Format suggestions with proper indentation and dimmed color
fn format_suggestions(suggestions: &[String], base_color: (u8, u8, u8)) -> String {
    if suggestions.is_empty() {
        String::new()
    } else {
        let dimmer_color = dim_color(base_color.0, base_color.1, base_color.2, 0.7);
        let suggestion_lines: Vec<String> = suggestions
            .iter()
            .map(|s| format!("    👉 {}", format_args!("{}", s).fg(dimmer_color)))
            .collect();
        format!("\n{}", suggestion_lines.join("\n"))
    }
}

/// Find the longest line in a document by token analysis
fn find_longest_doc_line(toks: &[Token]) -> usize {
    let mut longest_len_chars = 0;
    let mut curr_len_chars = 0;
    let mut current_line_start_tok_idx = 0;

    for (idx, tok) in toks.iter().enumerate() {
        if matches!(tok.kind, harper_core::TokenKind::Newline(_))
            || matches!(tok.kind, harper_core::TokenKind::ParagraphBreak)
        {
            if curr_len_chars > longest_len_chars {
                longest_len_chars = curr_len_chars;
            }
            curr_len_chars = 0;
            current_line_start_tok_idx = idx + 1;
        } else if matches!(tok.kind, harper_core::TokenKind::Unlintable) {
            // TODO would be more accurate to scan for \n in tok.get_ch(src)
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

/// Final summary report for all processed files
pub fn final_report(
    dialect: harper_core::Dialect,
    batch_mode: bool,
    all_lint_kinds: HashMap<LintKind, usize>,
    all_rules: HashMap<String, usize>,
    all_lint_kind_rule_pairs: HashMap<(LintKind, String), usize>,
    all_spellos: HashMap<String, usize>,
    color: bool,
) {
    // The stats summary of all inputs that we only do when there are multiple inputs.
    if batch_mode {
        let mut all_files_lint_kind_counts_vec: Vec<(LintKind, _)> =
            all_lint_kinds.into_iter().collect();
        all_files_lint_kind_counts_vec
            .sort_by_key(|(lk, count)| (std::cmp::Reverse(*count), lk.to_string()));

        let lint_kind_counts: Vec<(Option<String>, String)> = all_files_lint_kind_counts_vec
            .into_iter()
            .map(|(lint_kind, c)| {
                let (r, g, b) = rgb_for_lint_kind(Some(&lint_kind));
                (
                    Some(format!("\x1b[38;2;{r};{g};{b}m")),
                    format!("[{lint_kind}: {c}]"),
                )
            })
            .collect();

        if !lint_kind_counts.is_empty() {
            println!("All files lint kinds:");
            print_formatted_items(lint_kind_counts, color);
        }

        let mut all_files_rule_name_counts_vec: Vec<_> = all_rules.into_iter().collect();
        all_files_rule_name_counts_vec
            .sort_by_key(|(rule_name, count)| (std::cmp::Reverse(*count), rule_name.to_string()));

        let rule_name_counts: Vec<(Option<String>, String)> = all_files_rule_name_counts_vec
            .into_iter()
            .map(|(rule_name, count)| (None, format!("({rule_name}: {count})")))
            .collect();

        if !rule_name_counts.is_empty() {
            println!("All files rule names:");
            print_formatted_items(rule_name_counts, color);
        }
    }

    // The stats summary of all pairs of lint kind + rule name, whether there is only one input or multiple.
    let mut lint_kind_rule_pairs: Vec<_> = all_lint_kind_rule_pairs.into_iter().collect();
    lint_kind_rule_pairs.sort_by(|a, b| {
        let (a, b) = ((&a.0, &a.1), (&b.0, &b.1));
        b.1.cmp(a.1)
            .then_with(|| a.0.0.to_string().cmp(&b.0.0.to_string()))
            .then_with(|| a.0.1.cmp(&b.0.1))
    });

    // Format them using their colours
    let formatted_lint_kind_rule_pairs: Vec<(Option<String>, String)> = lint_kind_rule_pairs
        .into_iter()
        .map(|ele| {
            let (r, g, b) = rgb_for_lint_kind(Some(&ele.0.0));
            let ansi_prefix = format!("\x1b[38;2;{r};{g};{b}m");
            (
                Some(ansi_prefix),
                format!("«« {} {}·{} »»", ele.1, ele.0.0, ele.0.1),
            )
        })
        .collect();

    if !formatted_lint_kind_rule_pairs.is_empty() {
        // Print them with line wrapping
        print_formatted_items(formatted_lint_kind_rule_pairs, color);
    }

    if !all_spellos.is_empty() {
        // Group by lowercase spelling while preserving original case and counts
        let mut grouped: HashMap<String, Vec<(String, usize)>> = HashMap::new();
        for (spelling, count) in all_spellos {
            grouped
                .entry(spelling.to_lowercase())
                .or_default()
                .push((spelling, count));
        }

        // Create a vector of (lowercase_spelling, variants, total_count)
        let mut grouped_vec: Vec<_> = grouped
            .into_iter()
            .map(|(lower, variants)| {
                let total: usize = variants.iter().map(|(_, c)| c).sum();
                (lower, variants, total)
            })
            .collect();

        // Sort by total count (descending), then by lowercase spelling
        grouped_vec.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

        // Flatten the variants back out, but keep track of the group index for coloring
        let spelling_vec: Vec<(Option<String>, String)> = grouped_vec
            .into_iter()
            .enumerate()
            .flat_map(|(i, (_, variants, _))| {
                // Sort variants by count (descending) then by original spelling
                let mut variants = variants;
                variants.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

                // Choose colour based on group index (rotating through three colours)
                let (r, g, b) = match i % 3 {
                    0 => (180, 90, 150), // Magenta
                    1 => (90, 180, 90),  // Green
                    _ => (90, 150, 180), // Cyan
                };
                let ansi_color = format!("\x1b[38;2;{};{};{}m", r, g, b);

                variants.into_iter().map(move |(spelling, c)| {
                    (
                        Some(ansi_color.clone()),
                        format!("(\u{201c}{spelling}\u{201d}: {c})"),
                    )
                })
            })
            .collect();

        println!("All files Spelling::SpellCheck (For dialect: {})", dialect);
        print_formatted_items(spelling_vec, color);
    }
}

pub fn rgb_for_lint_kind(olk: Option<&LintKind>) -> (u8, u8, u8) {
    olk.and_then(|lk| {
        lint_kind_to_rgb()
            .iter()
            .find(|(k, _)| k == lk)
            .map(|(_, color)| *color)
    })
    .unwrap_or((0, 0, 0))
}

pub fn print_formatted_items(
    items: impl IntoIterator<Item = (Option<String>, String)>,
    color: bool,
) {
    let mut first_on_line = true;
    let mut len_so_far = 0;

    for (ansi, text) in items {
        let text_len = text.len();

        let mut len_to_add = !first_on_line as usize + text_len;

        let mut before = "";
        if len_so_far + len_to_add > 120 {
            before = "\n";
            len_to_add -= 1; // no space before the first item
            len_so_far = 0;
        } else if !first_on_line {
            before = " ";
        }

        let (set, reset): (&str, &str) = if color {
            if let Some(prefix) = ansi.as_ref() {
                (prefix.as_str(), "\x1b[0m")
            } else {
                ("", "")
            }
        } else {
            ("", "")
        };
        print!("{}{}{}{}", before, set, text, reset);
        len_so_far += len_to_add;
        first_on_line = false;
    }
}

/// Build and print a rich (Ariadne) report for a single input
pub fn build_rich_report(
    input_identifier: &str,
    named_lints: &std::collections::BTreeMap<String, Vec<harper_core::linting::Lint>>,
    source: &str,
    lint_count_after: usize,
    doc: &Document,
    batch_mode: bool,
) {
    if lint_count_after == 0 {
        return;
    }

    // Check if we should use rich format based on line length
    const MAX_LINE_LEN: usize = 150;
    if !batch_mode {
        // Always use rich for single files
    } else {
        let longest = find_longest_doc_line(doc.get_tokens());
        if longest > MAX_LINE_LEN {
            println!(
                "{}: Longest line exceeds max line length for rich format, using brief counts",
                input_identifier
            );
            return;
        }
    }

    let primary_color = Color::Magenta;

    let mut report_builder = Report::build(ReportKind::Advice, (input_identifier, 0..0));

    for (rule_name, lints) in named_lints {
        for lint in lints {
            let (r, g, b) = rgb_for_lint_kind(Some(&lint.lint_kind));
            report_builder = report_builder.with_label(
                Label::new((input_identifier, lint.span.into()))
                    .with_message({
                        let suggestions: Vec<String> =
                            lint.suggestions.iter().map(|s| format!("{s}")).collect();
                        let suggestion_text = format_suggestions(&suggestions, (r, g, b));

                        format!(
                            "{} {}: {}{}",
                            format_args!("[{}::{}]", lint.lint_kind, rule_name)
                                .fg(Color::Rgb(r, g, b)),
                            format_args!("(pri {})", lint.priority).fg(dim_color(r, g, b, 0.66)),
                            lint.message,
                            suggestion_text
                        )
                    })
                    .with_color(primary_color),
            );
        }
    }

    let report = report_builder.finish();
    report.print((input_identifier, Source::from(source))).ok();
}

// Note: This must be kept synchronized with:
// packages/lint-framework/src/lint/lintKindColor.ts
// packages/web/src/lib/lintKindColor.ts
// This can be removed when issue #1991 is resolved.
fn lint_kind_to_rgb() -> &'static [(LintKind, (u8, u8, u8))] {
    &[
        (LintKind::Agreement, (0x22, 0x8B, 0x22)),
        (LintKind::BoundaryError, (0x8B, 0x45, 0x13)),
        (LintKind::Capitalization, (0x54, 0x0D, 0x6E)),
        (LintKind::Eggcorn, (0xFF, 0x8C, 0x00)),
        (LintKind::Enhancement, (0x0E, 0xAD, 0x69)),
        (LintKind::Formatting, (0x7D, 0x3C, 0x98)),
        (LintKind::Grammar, (0x9B, 0x59, 0xB6)),
        (LintKind::Malapropism, (0xC7, 0x15, 0x85)),
        (LintKind::Miscellaneous, (0x3B, 0xCE, 0xAC)),
        (LintKind::Nonstandard, (0x00, 0x8B, 0x8B)),
        (LintKind::Punctuation, (0xD4, 0x85, 0x0F)),
        (LintKind::Readability, (0x2E, 0x8B, 0x57)),
        (LintKind::Redundancy, (0x46, 0x82, 0xB4)),
        (LintKind::Regionalism, (0xC0, 0x61, 0xCB)),
        (LintKind::Repetition, (0x00, 0xA6, 0x7C)),
        (LintKind::Spelling, (0xEE, 0x42, 0x66)),
        (LintKind::Style, (0xFF, 0xD2, 0x3F)),
        (LintKind::Typo, (0xFF, 0x6B, 0x35)),
        (LintKind::Usage, (0x1E, 0x90, 0xFF)),
        (LintKind::WordChoice, (0x22, 0x8B, 0x22)),
    ]
}

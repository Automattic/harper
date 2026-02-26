use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use anyhow::Context;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use hashbrown::HashMap;
use rayon::prelude::*;

use harper_core::{
    linting::{Lint, LintGroup, LintGroupConfig, LintKind},
    parsers::MarkdownOptions,
    spell::{Dictionary, MergedDictionary, MutableDictionary},
    weirpack::Weirpack,
    {Dialect, DictWordMetadata, Document, Token, TokenKind, remove_overlaps_map},
};

use crate::input::{
    AnyInput, InputTrait,
    multi_input::MultiInput,
    single_input::{SingleInput, SingleInputTrait, StdinInput},
};

/// Sync version of harper-ls/src/dictionary_io@load_dict
fn load_dict(path: &Path) -> anyhow::Result<MutableDictionary> {
    let str = fs::read_to_string(path)?;

    let mut dict = MutableDictionary::new();
    dict.extend_words(
        str.lines()
            .map(|l| (l.chars().collect::<Vec<_>>(), DictWordMetadata::default())),
    );

    Ok(dict)
}

fn load_weirpacks(inputs: &[SingleInput]) -> anyhow::Result<Vec<Weirpack>> {
    let mut packs = Vec::new();
    for input in inputs {
        let Some(file) = input.try_as_file_ref() else {
            anyhow::bail!(
                "Weirpack inputs must be files, got {}",
                input.get_identifier()
            );
        };

        let path = file.path();
        let bytes = fs::read(path)
            .with_context(|| format!("Failed to read weirpack {}", path.display()))?;
        let pack = Weirpack::from_bytes(&bytes)
            .with_context(|| format!("Failed to load weirpack {}", path.display()))?;
        packs.push(pack);
    }
    Ok(packs)
}

/// Path version of harper-ls/src/dictionary_io@file_dict_name
fn file_dict_name(path: &Path) -> PathBuf {
    let mut rewritten = String::new();

    for seg in path.components() {
        if !matches!(seg, Component::RootDir) {
            rewritten.push_str(&seg.as_os_str().to_string_lossy());
            rewritten.push('%');
        }
    }

    rewritten.into()
}

pub struct LintOptions {
    pub count: bool,
    pub ignore: Option<Vec<String>>,
    pub only: Option<Vec<String>>,
    pub keep_overlapping_lints: bool,
    pub dialect: Dialect,
    pub weirpack_inputs: Vec<SingleInput>,
    pub color: bool,
}
enum ReportStyle {
    FullAriadneLintReport,
    BriefCountsOnlyLintReport,
}

struct InputInfo<'a> {
    parent_input_id: &'a str,
    input: &'a AnyInput,
    color: bool,
}

struct InputJob {
    batch_mode: bool,
    parent_input_id: String,
    input: AnyInput,
}

impl InputInfo<'_> {
    fn format_path(&self) -> String {
        let child = self.input.get_identifier();
        if self.parent_input_id.is_empty() {
            child.into_owned()
        } else if self.color {
            format!("\x1b[33m{}/\x1b[0m{}", self.parent_input_id, child)
        } else {
            format!("{}/{}", self.parent_input_id, child)
        }
    }
}

pub fn lint(
    markdown_options: MarkdownOptions,
    curated_dictionary: Arc<dyn Dictionary>,
    mut inputs: Vec<AnyInput>,
    mut lint_options: LintOptions,
    user_dict_path: PathBuf,
    // TODO workspace_dict_path?
    file_dict_path: PathBuf,
) -> anyhow::Result<()> {
    let LintOptions {
        count,
        ref mut ignore,
        ref mut only,
        dialect,
        ref weirpack_inputs,
        ..
    } = lint_options;

    // Zero or more inputs, default to stdin if not provided
    if inputs.is_empty() {
        inputs.push(SingleInput::from(StdinInput).into());
    }

    let weirpacks = load_weirpacks(weirpack_inputs)?;

    // Filter out any rules from ignore/only lists that don't exist in the current config
    // Uses a cached config to avoid expensive linter initialization
    let mut config = LintGroupConfig::new_curated();
    for pack in &weirpacks {
        for rule in pack.rules.keys() {
            config.set_rule_enabled(rule, true);
        }
    }

    if let Some(only) = only {
        only.retain(|rule| {
            if !config.has_rule(rule) {
                eprintln!("Warning: Cannot enable unknown rule '{}'.", rule);
                return false;
            }
            true
        });
    }

    if let Some(ignore) = ignore {
        ignore.retain(|rule| {
            if !config.has_rule(rule) {
                eprintln!("Warning: Cannot disable unknown rule '{}'.", rule);
                return false;
            }
            true
        });
    }

    // Create merged dictionary with base dictionary
    let mut curated_plus_user_dict = MergedDictionary::new();
    curated_plus_user_dict.add_dictionary(Arc::new(curated_dictionary));

    let user_dict_msg = match load_dict(&user_dict_path) {
        Ok(user_dict) => {
            curated_plus_user_dict.add_dictionary(Arc::new(user_dict));
            "Using"
        }
        Err(_) => "There is no",
    };
    println!(
        "Note: {user_dict_msg} user dictionary at {}",
        user_dict_path.display()
    );

    // The lint stats for all files
    let mut all_lint_kinds: HashMap<LintKind, usize> = HashMap::new();
    let mut all_rules: HashMap<String, usize> = HashMap::new();
    let mut all_lint_kind_rule_pairs: HashMap<(LintKind, String), usize> = HashMap::new();
    let mut all_spellos: HashMap<String, usize> = HashMap::new();

    // Convert the 'count' flag into a ReportStyle enum
    let report_mode = match count {
        true => ReportStyle::BriefCountsOnlyLintReport,
        false => ReportStyle::FullAriadneLintReport,
    };

    let mut input_jobs = Vec::new();
    for user_input in inputs {
        if let Some(dir_input) = user_input
            .try_as_multi_ref()
            .and_then(MultiInput::try_as_dir_ref)
        {
            let mut file_entries: Vec<_> = dir_input.iter_files()?.collect();

            file_entries.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));

            for entry in file_entries.into_iter().map(SingleInput::from) {
                input_jobs.push(InputJob {
                    batch_mode: true,
                    parent_input_id: user_input.get_identifier().to_string(),
                    input: entry.into(),
                });
            }
        } else {
            input_jobs.push(InputJob {
                batch_mode: false,
                parent_input_id: String::new(),
                input: user_input.clone(),
            });
        }
    }

    let per_input_results = {
        let run_job = |job: InputJob| {
            let InputJob {
                batch_mode,
                parent_input_id,
                input,
            } = job;
            lint_one_input(
                // Common properties of harper-cli
                markdown_options,
                &curated_plus_user_dict,
                // Passed from the user for the `lint` subcommand
                &report_mode,
                &lint_options,
                &weirpacks,
                &file_dict_path,
                // Are we linting multiple inputs inside a directory?
                batch_mode,
                // The current input to be linted
                InputInfo {
                    parent_input_id: parent_input_id.as_str(),
                    input: &input,
                    color: lint_options.color,
                },
            )
        };

        if input_jobs.len() > 1 {
            input_jobs.into_par_iter().map(run_job).collect::<Vec<_>>()
        } else {
            input_jobs.into_iter().map(run_job).collect::<Vec<_>>()
        }
    };

    for lint_results in per_input_results {
        let lint_results = lint_results?;
        // Update the global stats
        for (kind, count) in lint_results.0 {
            *all_lint_kinds.entry(kind).or_insert(0) += count;
        }
        for (rule, count) in lint_results.1 {
            *all_rules.entry(rule).or_insert(0) += count;
        }
        for ((kind, rule), count) in lint_results.2 {
            *all_lint_kind_rule_pairs.entry((kind, rule)).or_insert(0) += count;
        }
        for (word, count) in lint_results.3 {
            *all_spellos.entry(word).or_insert(0) += count;
        }
    }

    final_report(
        dialect,
        true,
        all_lint_kinds,
        all_rules,
        all_lint_kind_rule_pairs,
        all_spellos,
        lint_options.color,
    );

    process::exit(1);
}

type LintKindCount = HashMap<LintKind, usize>;
type LintRuleCount = HashMap<String, usize>;
type LintKindRulePairCount = HashMap<(LintKind, String), usize>;
type SpelloCount = HashMap<String, usize>;

struct FullInputInfo<'a> {
    input: InputInfo<'a>,
    doc: Document,
    source: Cow<'a, str>,
}

#[allow(clippy::too_many_arguments)]
fn lint_one_input(
    // Common properties of harper-cli
    markdown_options: MarkdownOptions,
    curated_plus_user_dict: &MergedDictionary,
    report_mode: &ReportStyle,
    // Options passed from the user specific to the `lint` subcommand
    lint_options: &LintOptions,
    weirpacks: &[Weirpack],
    file_dict_path: &Path,
    // Are we linting multiple inputs?
    batch_mode: bool,
    // For the current input
    current: InputInfo,
) -> anyhow::Result<(
    LintKindCount,
    LintRuleCount,
    LintKindRulePairCount,
    SpelloCount,
)> {
    let LintOptions {
        count: _,
        ignore,
        only,
        keep_overlapping_lints,
        dialect,
        weirpack_inputs: _,
        color: _,
    } = lint_options;

    let mut lint_kinds: HashMap<LintKind, usize> = HashMap::new();
    let mut lint_rules: HashMap<String, usize> = HashMap::new();
    let mut lint_kind_rule_pairs: HashMap<(LintKind, String), usize> = HashMap::new();
    let mut spellos: HashMap<String, usize> = HashMap::new();

    if let Some(single_input) = current.input.try_as_single_ref() {
        // Create a new merged dictionary for this input.
        let mut merged_dictionary = curated_plus_user_dict.clone();

        // If processing a file, try to load its per-file dictionary
        if let Some(file) = single_input.try_as_file_ref() {
            let dict_path = file_dict_path.join(file_dict_name(file.path()));
            if let Ok(file_dictionary) = load_dict(&dict_path) {
                merged_dictionary.add_dictionary(Arc::new(file_dictionary));
                println!(
                    "{}: Note: Using per-file dictionary: {}",
                    current.format_path(),
                    dict_path.display()
                );
            }
        }

        match single_input.load(markdown_options, &merged_dictionary) {
            Err(err) => eprintln!("{}", err),
            Ok((doc, source)) => {
                // Create the Lint Group from which we will lint this input, using the combined dictionary and the specified dialect
                let mut lint_group = LintGroup::new_curated(merged_dictionary.into(), *dialect);

                for pack in weirpacks {
                    let mut pack_group = pack.to_lint_group()?;
                    lint_group.merge_from(&mut pack_group);
                }

                // Turn specified rules on or off
                configure_lint_group(&mut lint_group, only, ignore);

                // Run the linter, getting back a map of rule name -> lints
                let mut named_lints = lint_group.organized_lints(&doc);

                // Lint counts, for brief reporting
                let lint_count_before = named_lints.values().map(|v| v.len()).sum::<usize>();
                if !keep_overlapping_lints {
                    remove_overlaps_map(&mut named_lints);
                }
                let lint_count_after = named_lints.values().map(|v| v.len()).sum::<usize>();

                // Extract the lint kinds and rules etc. for reporting
                (lint_kinds, lint_rules) = count_lint_kinds_and_rules(&named_lints);
                lint_kind_rule_pairs = collect_lint_kind_rule_pairs(&named_lints);
                spellos = collect_spellos(&named_lints, doc.get_source());

                single_input_report(
                    &FullInputInfo {
                        input: InputInfo {
                            parent_input_id: current.parent_input_id,
                            input: current.input,
                            color: current.color,
                        },
                        doc,
                        source,
                    },
                    // Linting results of this input
                    &named_lints,
                    (lint_count_before, lint_count_after),
                    &lint_kinds,
                    &lint_rules,
                    // Reporting arguments
                    batch_mode,
                    report_mode,
                );
            }
        }
    }

    Ok((lint_kinds, lint_rules, lint_kind_rule_pairs, spellos))
}

fn configure_lint_group(
    lint_group: &mut LintGroup,
    only: &Option<Vec<String>>,
    ignore: &Option<Vec<String>>,
) {
    if let Some(rules) = only {
        lint_group.set_all_rules_to(Some(false));
        rules
            .iter()
            .for_each(|rule| lint_group.config.set_rule_enabled(rule, true));
    }

    if let Some(rules) = ignore {
        rules
            .iter()
            .for_each(|rule| lint_group.config.set_rule_enabled(rule, false));
    }
}

fn count_lint_kinds_and_rules(
    named_lints: &BTreeMap<String, Vec<Lint>>,
) -> (HashMap<LintKind, usize>, HashMap<String, usize>) {
    let mut kinds = HashMap::new();
    let mut rules = HashMap::new();

    for (rule_name, lints) in named_lints {
        lints
            .iter()
            .for_each(|lint| *kinds.entry(lint.lint_kind).or_insert(0) += 1);

        if !lints.is_empty() {
            *rules.entry(rule_name.to_string()).or_insert(0) += lints.len();
        }
    }

    (kinds, rules)
}

fn collect_lint_kind_rule_pairs(
    named_lints: &BTreeMap<String, Vec<Lint>>,
) -> HashMap<(LintKind, String), usize> {
    let mut pairs = HashMap::new();

    for (rule_name, lints) in named_lints {
        for lint in lints {
            pairs
                .entry((lint.lint_kind, rule_name.to_string()))
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    pairs
}

fn collect_spellos(
    named_lints: &BTreeMap<String, Vec<Lint>>,
    source: &[char],
) -> HashMap<String, usize> {
    named_lints
        .get("SpellCheck")
        .into_iter()
        .flatten()
        .map(|lint| lint.span.get_content_string(source))
        .fold(HashMap::new(), |mut acc, spello| {
            *acc.entry(spello).or_insert(0) += 1;
            acc
        })
}

fn single_input_report(
    // Properties of the current input
    input_info: &FullInputInfo,
    // Linting results of this input
    named_lints: &BTreeMap<String, Vec<Lint>>,
    lint_count: (usize, usize),
    lint_kinds: &HashMap<LintKind, usize>,
    lint_rules: &HashMap<String, usize>,
    // Reporting parameters
    batch_mode: bool, // If true, we are processing multiple files, which affects how we report
    report_mode: &ReportStyle,
) {
    let FullInputInfo { input, doc, source } = input_info;
    let (lint_count_before, lint_count_after) = lint_count;
    // The Ariadne report works poorly for files with very long lines, so suppress it unless only processing one file
    const MAX_LINE_LEN: usize = 150;

    let mut report_mode = report_mode;
    let longest = find_longest_doc_line(doc.get_tokens());

    if batch_mode
        && longest > MAX_LINE_LEN
        && matches!(report_mode, ReportStyle::FullAriadneLintReport)
    {
        report_mode = &ReportStyle::BriefCountsOnlyLintReport;
        println!(
            "{}: Longest line: {longest} exceeds max line length: {MAX_LINE_LEN}",
            input.format_path()
        );
    }

    // Report the number of lints no matter what report mode we are in
    println!(
        "{}: {}",
        input.format_path(),
        match (lint_count_before, lint_count_after) {
            (0, _) => "No lints found".to_string(),
            (before, after) if before != after =>
                format!("{before} lints before overlap removal, {after} after"),
            (before, _) => format!("{before} lints"),
        }
    );

    // If we are in Ariadne mode, print the report
    if matches!(report_mode, ReportStyle::FullAriadneLintReport) {
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
            // TODO would be more accurate to scan for \n in the tok.span.get_content(src)
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

fn final_report(
    dialect: Dialect,
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

fn rgb_for_lint_kind(olk: Option<&LintKind>) -> (u8, u8, u8) {
    olk.and_then(|lk| {
        lint_kind_to_rgb()
            .iter()
            .find(|(k, _)| k == lk)
            .map(|(_, color)| *color)
    })
    .unwrap_or((0, 0, 0))
}

fn print_formatted_items(items: impl IntoIterator<Item = (Option<String>, String)>, color: bool) {
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
    println!();
}

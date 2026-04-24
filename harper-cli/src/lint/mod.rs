pub mod rich_format;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use hashbrown::HashMap;
use rayon::prelude::*;
use serde::Serialize;

use harper_core::{
    linting::{FlatConfig, Lint, LintGroup, LintKind},
    parsers::MarkdownOptions,
    spell::{Dictionary, MergedDictionary, MutableDictionary},
    weirpack::Weirpack,
    {Dialect, DictWordMetadata, Document, remove_overlaps_map},
};

use crate::input::{
    AnyInput, InputTrait,
    multi_input::MultiInput,
    single_input::{SingleInput, SingleInputTrait, StdinInput},
};

use crate::lint::rich_format::{
    build_rich_report, final_report, print_formatted_items, rgb_for_lint_kind,
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

/// Output format for lint results.
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default, PartialEq)]
pub enum OutputFormat {
    /// Rich output with source context (Ariadne reports).
    #[default]
    #[clap(name = "rich")]
    Rich,
    /// Structured JSON output.
    #[clap(name = "json")]
    Json,
    /// One line per lint, no source context.
    #[clap(name = "compact")]
    Compact,
}

pub struct LintOptions {
    pub count: bool,
    pub ignore: Option<Vec<String>>,
    pub only: Option<Vec<String>>,
    pub keep_overlapping_lints: bool,
    pub dialect: Dialect,
    pub weirpack_inputs: Vec<SingleInput>,
    pub color: bool,
    pub format: OutputFormat,
}

enum ReportStyle {
    RichStyle,
    BriefCountsOnly,
    Json,
    Compact,
}

#[derive(Serialize)]
struct JsonFileResult {
    file: String,
    lint_count: usize,
    lints: Vec<JsonLint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct JsonLint {
    rule: String,
    kind: String,
    span: JsonSpan,
    line: usize,
    column: usize,
    message: String,
    priority: u8,
    suggestions: Vec<String>,
    matched_text: String,
}

/// Span offsets in characters (not bytes).
#[derive(Serialize)]
struct JsonSpan {
    char_start: usize,
    char_end: usize,
}

/// Convert a character index into a 1-based (line, column) pair.
fn char_index_to_line_col(source: &[char], index: usize) -> (usize, usize) {
    let before = &source[..index.min(source.len())];
    let line = before.iter().filter(|&&c| c == '\n').count() + 1;
    let col = before.iter().rev().take_while(|&&c| c != '\n').count() + 1;
    (line, col)
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
    /// Path without ANSI escapes, for machine-readable output.
    fn plain_path(&self) -> String {
        let child = self.input.get_identifier();
        if self.parent_input_id.is_empty() {
            child.into_owned()
        } else {
            format!("{}/{}", self.parent_input_id, child)
        }
    }

    fn format_path(&self) -> String {
        if self.color {
            let child = self.input.get_identifier();
            if self.parent_input_id.is_empty() {
                child.into_owned()
            } else {
                format!("\x1b[33m{}/\x1b[0m{}", self.parent_input_id, child)
            }
        } else {
            self.plain_path()
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
    let mut config = FlatConfig::new_curated();
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
    eprintln!(
        "Note: {user_dict_msg} user dictionary at {}",
        user_dict_path.display()
    );

    // The lint stats for all files
    let mut all_lint_kinds: HashMap<LintKind, usize> = HashMap::new();
    let mut all_rules: HashMap<String, usize> = HashMap::new();
    let mut all_lint_kind_rule_pairs: HashMap<(LintKind, String), usize> = HashMap::new();
    let mut all_spellos: HashMap<String, usize> = HashMap::new();

    // Derive the report style from --format and --count
    let report_mode = match (lint_options.format, count) {
        (OutputFormat::Json, _) => ReportStyle::Json,
        (OutputFormat::Compact, _) => ReportStyle::Compact,
        (OutputFormat::Rich, true) => ReportStyle::BriefCountsOnly,
        (OutputFormat::Rich, false) => ReportStyle::RichStyle,
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

    let mut json_results: Vec<JsonFileResult> = Vec::new();

    for lint_results in per_input_results {
        let lint_results = lint_results?;
        // Update the global stats
        for (kind, count) in lint_results.lint_kinds {
            *all_lint_kinds.entry(kind).or_insert(0) += count;
        }
        for (rule, count) in lint_results.lint_rules {
            *all_rules.entry(rule).or_insert(0) += count;
        }
        for ((kind, rule), count) in lint_results.lint_kind_rule_pairs {
            *all_lint_kind_rule_pairs.entry((kind, rule)).or_insert(0) += count;
        }
        for (word, count) in lint_results.spellos {
            *all_spellos.entry(word).or_insert(0) += count;
        }
        if let Some(json) = lint_results.json {
            json_results.push(json);
        }
    }

    let has_lints = !all_lint_kinds.is_empty();

    match report_mode {
        ReportStyle::Json => {
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
        ReportStyle::Compact => {}
        _ => {
            final_report(
                dialect,
                true,
                all_lint_kinds,
                all_rules,
                all_lint_kind_rule_pairs,
                all_spellos,
                lint_options.color,
            );
        }
    }

    if has_lints {
        anyhow::bail!("Lints were found");
    }

    Ok(())
}

struct LintOneResult {
    lint_kinds: HashMap<LintKind, usize>,
    lint_rules: HashMap<String, usize>,
    lint_kind_rule_pairs: HashMap<(LintKind, String), usize>,
    spellos: HashMap<String, usize>,
    json: Option<JsonFileResult>,
}

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
) -> anyhow::Result<LintOneResult> {
    let LintOptions {
        count: _,
        ignore,
        only,
        keep_overlapping_lints,
        dialect,
        weirpack_inputs: _,
        color: _,
        format: _,
    } = lint_options;

    let mut lint_kinds: HashMap<LintKind, usize> = HashMap::new();
    let mut lint_rules: HashMap<String, usize> = HashMap::new();
    let mut lint_kind_rule_pairs: HashMap<(LintKind, String), usize> = HashMap::new();
    let mut spellos: HashMap<String, usize> = HashMap::new();
    let mut json: Option<JsonFileResult> = None;

    if let Some(single_input) = current.input.try_as_single_ref() {
        // Create a new merged dictionary for this input.
        let mut merged_dictionary = curated_plus_user_dict.clone();

        // If processing a file, try to load its per-file dictionary
        if let Some(file) = single_input.try_as_file_ref() {
            let dict_path = file_dict_path.join(file_dict_name(file.path()));
            if let Ok(file_dictionary) = load_dict(&dict_path) {
                merged_dictionary.add_dictionary(Arc::new(file_dictionary));
                eprintln!(
                    "{}: Note: Using per-file dictionary: {}",
                    current.format_path(),
                    dict_path.display()
                );
            }
        }

        match single_input.load(markdown_options, &merged_dictionary) {
            Err(err) => {
                eprintln!("{}", err);
                if matches!(report_mode, ReportStyle::Json) {
                    json = Some(JsonFileResult {
                        file: current.plain_path(),
                        lint_count: 0,
                        lints: vec![],
                        error: Some(err.to_string()),
                    });
                }
            }
            Ok((doc, source)) => {
                // Create the Lint Group from which we will lint this input, using the combined dictionary and the specified dialect
                let mut lint_group = LintGroup::new_curated(merged_dictionary.into(), *dialect);

                for pack in weirpacks {
                    let pack_group = pack.to_lint_group()?;
                    lint_group.merge_from(pack_group);
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

                // Build JSON result if in JSON mode
                if matches!(report_mode, ReportStyle::Json) {
                    let file = current.plain_path();
                    let source_chars = doc.get_source();
                    let mut lints = Vec::new();

                    for (rule_name, rule_lints) in &named_lints {
                        for lint in rule_lints {
                            let (line, column) =
                                char_index_to_line_col(source_chars, lint.span.start);
                            let matched_text = lint.get_str(source_chars);
                            let suggestions: Vec<String> =
                                lint.suggestions.iter().map(|s| format!("{s}")).collect();
                            lints.push(JsonLint {
                                rule: rule_name.clone(),
                                kind: lint.lint_kind.to_string(),
                                span: JsonSpan {
                                    char_start: lint.span.start,
                                    char_end: lint.span.end,
                                },
                                line,
                                column,
                                message: lint.message.clone(),
                                priority: lint.priority,
                                suggestions,
                                matched_text,
                            });
                        }
                    }

                    json = Some(JsonFileResult {
                        file,
                        lint_count: lint_count_after,
                        lints,
                        error: None,
                    });
                }

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

    Ok(LintOneResult {
        lint_kinds,
        lint_rules,
        lint_kind_rule_pairs,
        spellos,
        json,
    })
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

    // Have all rules been disabled somehow?
    if !lint_group
        .iter_keys()
        .any(|rule| lint_group.config.is_rule_enabled(rule))
    {
        eprintln!("Warning: No rules are enabled.");
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
        .map(|lint| lint.get_str(source))
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

    // If we are in rich mode, print report
    if matches!(report_mode, ReportStyle::RichStyle) {
        build_rich_report(
            &input.input.get_identifier(),
            named_lints,
            source,
            lint_count_after,
            doc,
            batch_mode,
        );
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

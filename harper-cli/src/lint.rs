use std::{
    borrow::Cow,
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use hashbrown::HashMap;
use rayon::prelude::*;
use serde::Serialize;

use harper_core::{
    linting::{FlatConfig, LintKind},
    parsers::MarkdownOptions,
    spell::{Dictionary, MergedDictionary, MutableDictionary},
    weirpack::Weirpack,
    {Dialect, DictWordMetadata, Document},
};

use crate::input::{
    AnyInput, InputTrait,
    multi_input::MultiInput,
    single_input::{SingleInput, StdinInput},
};
use crate::lint_engine::lint_one_input;

/// Sync version of harper_dictionary_wordlist::load_dict.
pub fn load_dict(path: &Path) -> anyhow::Result<MutableDictionary> {
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

/// Path version of harper-ls file dictionary name rewriting.
pub fn file_dict_name(path: &Path) -> PathBuf {
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
    Default,
    /// Structured JSON output.
    Json,
    /// One line per lint, no source context.
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
    pub quiet: bool,
}

pub enum ReportStyle {
    FullAriadne,
    BriefCountsOnly,
    Json,
    Compact,
}

#[derive(Serialize)]
pub struct JsonFileResult {
    pub file: String,
    pub lint_count: usize,
    pub lints: Vec<JsonLint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct JsonLint {
    pub rule: String,
    pub kind: String,
    pub span: JsonSpan,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub priority: u8,
    pub suggestions: Vec<String>,
    pub matched_text: String,
}

/// Span offsets in characters (not bytes).
#[derive(Serialize)]
pub struct JsonSpan {
    pub char_start: usize,
    pub char_end: usize,
}

pub struct InputInfo<'a> {
    pub parent_input_id: &'a str,
    pub input: &'a AnyInput,
    pub color: bool,
}

struct InputJob {
    batch_mode: bool,
    parent_input_id: String,
    input: AnyInput,
}

impl InputInfo<'_> {
    /// Path without ANSI escapes, for machine-readable output.
    pub fn plain_path(&self) -> String {
        let child = self.input.get_identifier();
        if self.parent_input_id.is_empty() {
            child.into_owned()
        } else {
            format!("{}/{}", self.parent_input_id, child)
        }
    }

    pub fn format_path(&self) -> String {
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
        (OutputFormat::Default, true) => ReportStyle::BriefCountsOnly,
        (OutputFormat::Default, false) => ReportStyle::FullAriadne,
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

pub struct LintOneResult {
    pub lint_kinds: HashMap<LintKind, usize>,
    pub lint_rules: HashMap<String, usize>,
    pub lint_kind_rule_pairs: HashMap<(LintKind, String), usize>,
    pub spellos: HashMap<String, usize>,
    pub json: Option<JsonFileResult>,
}

pub struct FullInputInfo<'a> {
    pub input: InputInfo<'a>,
    pub doc: Document,
    pub source: Cow<'a, str>,
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
    println!();
}

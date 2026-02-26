#![doc = include_str!("../README.md")]

use harper_core::spell::{Dictionary, FstDictionary, MutableDictionary, WordId};
use hashbrown::HashMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
// use std::sync::Arc;
use std::{fs, process};

use anyhow::anyhow;
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use dirs::{config_dir, data_local_dir};
use harper_core::linting::LintGroup;
use harper_core::parsers::{IsolateEnglish, MarkdownOptions};
use harper_core::weir::WeirLinter;
use harper_core::{
    CharStringExt, Dialect, DictWordMetadata, OrthFlags, Span, TokenKind, TokenStringExt,
};
#[cfg(feature = "training")]
use harper_pos_utils::{BrillChunker, BrillTagger, BurnChunkerCpu};

use harper_stats::Stats;
use serde::Serialize;
use serde_json::Value;

mod input;
use input::{
    AnyInput, InputTrait,
    single_input::{SingleInput, SingleInputOptionExt, SingleInputTrait},
};

mod annotate;
use annotate::AnnotationType;

mod lint;
use crate::lint::lint;
use lint::LintOptions;

/// A debugging tool for the Harper grammar checker.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Disable colored output.
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Args,
}

#[derive(clap::Subcommand)]
enum Args {
    /// Lint provided documents.
    Lint {
        /// The text or file you wish to grammar check. If not provided, it will be read from
        /// standard input.
        inputs: Vec<AnyInput>,
        /// Whether to merely print out the number of errors encountered,
        /// without further details.
        #[arg(short, long)]
        count: bool,
        /// Restrict linting to only a specific set of rules.
        /// If omitted, `harper-cli` will run every rule.
        #[arg(long, value_delimiter = ',')]
        ignore: Option<Vec<String>>,
        /// Restrict linting to only a specific set of rules.
        /// If omitted, `harper-cli` will run every rule.
        #[arg(long, value_delimiter = ',')]
        only: Option<Vec<String>>,
        /// Overlapping lints are removed by default. This option disables that behavior.
        #[arg(short = 'o', long)]
        keep_overlapping_lints: bool,
        /// Specify the dialect. Common synonyms, abbreviations, and codes are supported.
        #[arg(short, long, default_value = "us")]
        dialect: String,
        /// Path to the user dictionary.
        #[arg(short, long, default_value = config_dir().unwrap().join("harper-ls/dictionary.txt").into_os_string())]
        user_dict_path: PathBuf,
        /// Path to the directory for file-local dictionaries.
        #[arg(short, long, default_value = data_local_dir().unwrap().join("harper-ls/file_dictionaries/").into_os_string())]
        file_dict_path: PathBuf,
        /// Path to a Weirpack file to load. May be supplied multiple times.
        #[arg(long, value_name = "WEIRPACK")]
        weirpacks: Vec<SingleInput>,
    },
    /// Parse a provided document and print the detected symbols.
    Parse {
        /// The text or file you wish to parse. If not provided, it will be read from standard
        /// input.
        input: Option<SingleInput>,
    },
    /// Parse a provided document and show the spans of the detected tokens.
    Spans {
        /// The file or text for which you wish to display the spans. If not provided, it will be
        /// read from standard input.
        input: Option<SingleInput>,
        /// Include newlines in the output
        #[arg(short, long)]
        include_newlines: bool,
    },
    /// Parse and annotate a provided document.
    Annotate {
        /// The text or file you wish to parse. If not provided, it will be read from standard
        /// input.
        input: Option<SingleInput>,
        /// How the document should be annotated.
        #[arg(short, long, value_enum, default_value_t = AnnotationType::Upos)]
        annotation_type: AnnotationType,
        /// Attempt to detect and ignore non-English spans of text.
        #[arg(short, long)]
        isolate_english: bool,
    },
    /// Get the metadata associated with one or more words.
    Metadata {
        words: Vec<String>,
        /// Only show the part-of-speech flags and emojis, not the full JSON
        #[arg(short, long)]
        brief: bool,
    },
    /// Get all the forms of a word using the affixes.
    Forms { line: String },
    /// Emit a decompressed, line-separated list of the words in Harper's dictionary.
    Words,
    /// Summarize a lint record
    SummarizeLintRecord { file: PathBuf },
    /// Print the default config with descriptions.
    Config,
    /// Print a list of all the words in a document, sorted by frequency.
    MineWords {
        /// The document to mine words from.
        input: Option<SingleInput>,
    },
    #[cfg(feature = "training")]
    TrainBrillTagger {
        #[arg(short, long, default_value = "1.0")]
        candidate_selection_chance: f32,
        /// The path to write the final JSON model file to.
        output: PathBuf,
        /// The number of epochs (and patch rules) to train.
        epochs: usize,
        /// Path to a `.conllu` dataset to train on.
        #[arg(num_args = 1..)]
        datasets: Vec<PathBuf>,
    },
    #[cfg(feature = "training")]
    TrainBrillChunker {
        #[arg(short, long, default_value = "1.0")]
        candidate_selection_chance: f32,
        /// The path to write the final JSON model file to.
        output: PathBuf,
        /// The number of epochs (and patch rules) to train.
        epochs: usize,
        /// Path to a `.conllu` dataset to train on.
        #[arg(num_args = 1..)]
        datasets: Vec<PathBuf>,
    },
    #[cfg(feature = "training")]
    TrainBurnChunker {
        #[arg(short, long)]
        lr: f64,
        // The number of embedding dimensions
        #[arg(long)]
        dim: usize,
        /// The path to write the final  model file to.
        #[arg(short, long)]
        output: PathBuf,
        /// The number of epochs to train.
        #[arg(short, long)]
        epochs: usize,
        /// The dropout probability
        #[arg(long)]
        dropout: f32,
        #[arg(short, long)]
        test_file: PathBuf,
        #[arg(num_args = 1..)]
        datasets: Vec<PathBuf>,
    },
    /// Print harper-core version.
    CoreVersion,
    /// Rename a flag in the dictionary and affixes.
    RenameFlag {
        /// The old flag.
        old: String,
        /// The new flag.
        new: String,
        /// The directory containing the dictionary and affixes.
        dir: PathBuf,
    },
    /// Audit the `dictionary.dict` file.
    AuditDictionary {
        /// The directory containing the dictionary and affixes.
        dir: PathBuf,
    },
    /// Emit a decompressed, line-separated list of the compounds in Harper's dictionary.
    /// As long as there's either an open or hyphenated spelling.
    Compounds,
    /// Emit a decompressed, line-separated list of the words in Harper's dictionary
    /// which occur in more than one lettercase variant.    
    CaseVariants,
    /// Emit a list of each noun phrase contained within the input
    NominalPhrases {
        /// The text or file to analyze. If not provided, it will be read from standard input.
        input: Option<SingleInput>,
    },
    /// Run the tests contained within a Weir file.
    Test {
        /// The location of the Weir file to test
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let color = !cli.no_color && std::env::var("NO_COLOR").is_err();
    if !color {
        yansi::disable();
    }

    let markdown_options = MarkdownOptions::default();
    let curated_dictionary = FstDictionary::curated();

    match cli.command {
        Args::Lint {
            inputs,
            count,
            ignore,
            only,
            keep_overlapping_lints,
            dialect: dialect_str,
            user_dict_path,
            file_dict_path,
            weirpacks,
        } => {
            let dialect = parse_dialect(&dialect_str)
                .map_err(|e| anyhow!("Invalid dialect '{}': {}", dialect_str, e))?;

            lint(
                markdown_options,
                curated_dictionary,
                inputs,
                LintOptions {
                    count,
                    ignore,
                    only,
                    keep_overlapping_lints,
                    dialect,
                    weirpack_inputs: weirpacks,
                    color,
                },
                user_dict_path,
                // TODO workspace_dict_path?
                file_dict_path,
            )
        }
        Args::Parse { input } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_read_from_stdin();

            // Load the file/text.
            let (doc, _) = input.load(markdown_options, &curated_dictionary)?;

            for token in doc.tokens() {
                let json = serde_json::to_string(&token)?;
                println!("{json}");
            }

            Ok(())
        }
        Args::Spans {
            input,
            include_newlines,
        } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_read_from_stdin();

            // Load the file/text.
            let (doc, source) = input.load(markdown_options, &curated_dictionary)?;

            let primary_color = Color::Blue;
            let secondary_color = Color::Magenta;
            let unlintable_color = Color::Red;
            let input_identifier = input.get_identifier();

            let mut report_builder = Report::build(
                ReportKind::Custom("Spans", primary_color),
                (&input_identifier, 0..0),
            );
            let mut color = primary_color;

            for token in doc.tokens().filter(|t| {
                include_newlines
                    || !matches!(t.kind, TokenKind::Newline(_) | TokenKind::ParagraphBreak)
            }) {
                report_builder = report_builder.with_label(
                    Label::new((&input_identifier, token.span.into()))
                        .with_message(format!("[{}, {})", token.span.start, token.span.end))
                        .with_color(if matches!(token.kind, TokenKind::Unlintable) {
                            unlintable_color
                        } else {
                            color
                        }),
                );

                // Alternate colors so spans are clear
                color = if color == primary_color {
                    secondary_color
                } else {
                    primary_color
                };
            }

            let report = report_builder.finish();
            report.print((&input_identifier, Source::from(source)))?;

            Ok(())
        }
        Args::Annotate {
            input,
            annotation_type,
            isolate_english,
        } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_read_from_stdin();

            let parser = if isolate_english {
                Box::new(IsolateEnglish::new(
                    input.get_parser(markdown_options),
                    &curated_dictionary,
                ))
            } else {
                input.get_parser(markdown_options)
            };

            // Load the file/text.
            let (doc, source) = input.load_with_parser(&parser, &curated_dictionary)?;

            let input_identifier = input.get_identifier();

            annotation_type
                .build_report(
                    &doc,
                    &input_identifier,
                    &annotation_type.get_title_with_tags(if isolate_english {
                        &["Isolate english"]
                    } else {
                        &[]
                    }),
                )
                .print((&*input_identifier, Source::from(source)))?;

            Ok(())
        }
        Args::Words => {
            let mut word_str = String::new();

            for word in curated_dictionary.words_iter() {
                word_str.clear();
                word_str.extend(word);

                println!("{word_str:?}");
            }

            Ok(())
        }
        Args::Metadata { words, brief } => {
            type PosPredicate = fn(&DictWordMetadata) -> bool;

            const POS: &[(&str, PosPredicate)] = &[
                ("N📦", |m| m.is_noun() && !m.is_proper_noun()),
                ("O📛", DictWordMetadata::is_proper_noun),
                ("V🏃", DictWordMetadata::is_verb),
                ("J🌈", DictWordMetadata::is_adjective),
                ("R🤷", DictWordMetadata::is_adverb),
                ("C🔗", DictWordMetadata::is_conjunction),
                ("D👉", DictWordMetadata::is_determiner),
                ("P📥", |m| m.preposition),
                ("I👤", DictWordMetadata::is_pronoun),
            ];

            for word in words {
                let meta = curated_dictionary.get_word_metadata_str(&word);
                let (flags, emojis) = meta.as_ref().map_or_else(
                    || (String::new(), String::new()),
                    |md| {
                        POS.iter()
                            .filter(|&(_, pred)| pred(md))
                            .map(|(syms, _)| {
                                let mut ch = syms.chars();
                                (ch.next().unwrap(), ch.next().unwrap())
                            })
                            .unzip()
                    },
                );

                let json = brief.then(String::new).unwrap_or_else(|| {
                    format!("\n{}", serde_json::to_string_pretty(&meta).unwrap())
                });
                println!("{}: {} {}{}", word, flags, emojis, json);
            }
            Ok(())
        }
        Args::SummarizeLintRecord { file } => {
            let file = File::open(file)?;
            let mut reader = BufReader::new(file);
            let stats = Stats::read(&mut reader)?;

            let summary = stats.summarize();
            println!("{summary}");

            Ok(())
        }
        Args::Forms { line } => {
            let (word, annot) = line_to_parts(&line);

            let curated_word_list = include_str!("../../harper-core/dictionary.dict");
            let dict_lines = curated_word_list.split('\n');

            let mut entry_in_dict = None;

            // Check if the word is contained in the list.
            for dict_line in dict_lines {
                let (dict_word, dict_annot) = line_to_parts(dict_line);

                if dict_word == word {
                    entry_in_dict = Some((dict_word, dict_annot));
                    break;
                }
            }

            let summary = match &entry_in_dict {
                Some((dict_word, dict_annot)) => {
                    let mut status_summary = if dict_annot.is_empty() {
                        format!("'{dict_word}' is already in the dictionary but not annotated.")
                    } else {
                        format!(
                            "'{dict_word}' is already in the dictionary with annotation `{dict_annot}`."
                        )
                    };

                    if !annot.is_empty() {
                        if annot.as_str() != dict_annot.as_str() {
                            status_summary
                                .push_str("\n  Your annotations differ from the dictionary.\n");
                        } else {
                            status_summary
                                .push_str("\n  Your annotations are the same as the dictionary.\n");
                        }
                    }

                    status_summary
                }
                None => format!("'{word}' is not in the dictionary yet."),
            };

            println!("{summary}");

            if let Some((dict_word, dict_annot)) = &entry_in_dict {
                println!("Old, from the dictionary:");
                print_word_derivations(dict_word, dict_annot, &FstDictionary::curated());
            };

            if !annot.is_empty() {
                let rune_words = format!("1\n{line}");
                let dict = MutableDictionary::from_rune_files(
                    &rune_words,
                    include_str!("../../harper-core/annotations.json"),
                )?;

                println!("New, from you:");
                print_word_derivations(&word, &annot, &dict);
            }

            Ok(())
        }
        Args::Config => {
            #[derive(Serialize)]
            struct Config {
                default_value: bool,
                description: String,
            }

            let linter = LintGroup::new_curated(curated_dictionary, Dialect::American);

            let default_config: HashMap<String, bool> =
                serde_json::from_str(&serde_json::to_string(&linter.config).unwrap()).unwrap();

            // Use `BTreeMap` so output is sorted by keys.
            let mut configs = BTreeMap::new();
            for (key, desc) in linter.all_descriptions() {
                configs.insert(
                    key.to_owned(),
                    Config {
                        default_value: default_config[key],
                        description: desc.to_owned(),
                    },
                );
            }

            println!("{}", serde_json::to_string_pretty(&configs).unwrap());

            Ok(())
        }
        Args::MineWords { input } => {
            let input = input.unwrap_or_read_from_stdin();
            let (doc, _source) = input.load(MarkdownOptions::default(), &curated_dictionary)?;

            let mut words = HashMap::new();

            for word in doc.iter_words() {
                let chars = doc.get_span_content(&word.span);

                words
                    .entry(chars.to_lower())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
            }

            let mut words_ordered: Vec<(String, usize)> = words
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect();

            words_ordered.sort_by_key(|v| v.1);

            for (word, _) in words_ordered {
                println!("{word}");
            }

            Ok(())
        }
        Args::CoreVersion => {
            println!("harper-core v{}", harper_core::core_version());
            Ok(())
        }
        #[cfg(feature = "training")]
        Args::TrainBrillTagger {
            datasets: dataset,
            epochs,
            output,
            candidate_selection_chance,
        } => {
            let tagger = BrillTagger::train(&dataset, epochs, candidate_selection_chance);
            fs::write(output, serde_json::to_string_pretty(&tagger)?)?;

            Ok(())
        }
        #[cfg(feature = "training")]
        Args::TrainBrillChunker {
            datasets,
            epochs,
            output,
            candidate_selection_chance,
        } => {
            let chunker = BrillChunker::train(&datasets, epochs, candidate_selection_chance);
            fs::write(output, serde_json::to_string_pretty(&chunker)?)?;
            Ok(())
        }
        #[cfg(feature = "training")]
        Args::TrainBurnChunker {
            datasets,
            test_file,
            epochs,
            dropout,
            output,
            lr,
            dim: embed_dim,
        } => {
            let chunker =
                BurnChunkerCpu::train_cpu(&datasets, &test_file, embed_dim, dropout, epochs, lr);
            chunker.save_to(output);

            Ok(())
        }
        Args::RenameFlag { old, new, dir } => {
            let dict_path = dir.join("dictionary.dict");
            let affixes_path = dir.join("annotations.json");

            // Validate old and new flags are exactly one Unicode code point (Rust char)
            // And not characters used for the dictionary format
            const BAD_CHARS: [char; 3] = ['/', '#', ' '];

            // Then use it like this:
            if old.chars().count() != 1 || BAD_CHARS.iter().any(|&c| old.contains(c)) {
                return Err(anyhow!(
                    "Flags must be one Unicode code point, not / or # or space. Old flag '{old}' is {}",
                    old.chars().count()
                ));
            }
            if new.chars().count() != 1 || BAD_CHARS.iter().any(|&c| new.contains(c)) {
                return Err(anyhow!(
                    "Flags must be one Unicode code point, not / or # or space. New flag '{new}' is {}",
                    new.chars().count()
                ));
            }

            // Load and parse affixes
            let affixes_string = fs::read_to_string(&affixes_path)
                .map_err(|e| anyhow!("Failed to read annotations.json: {e}"))?;

            let affixes_json: Value = serde_json::from_str(&affixes_string)
                .map_err(|e| anyhow!("Failed to parse annotations.json: {e}"))?;

            // Get the nested "affixes" object
            let affixes_obj = &affixes_json
                .get("affixes")
                .and_then(Value::as_object)
                .ok_or_else(|| anyhow!("annotations.json does not contain 'affixes' object"))?;

            let properties_obj = &affixes_json
                .get("properties")
                .and_then(Value::as_object)
                .ok_or_else(|| anyhow!("annotations.json does not contain 'properties' object"))?;

            // Validate old flag exists and get its description
            let old_entry = affixes_obj
                .get(&old)
                .or_else(|| properties_obj.get(&old))
                .ok_or_else(|| anyhow!("Flag '{old}' not found in annotations.json"))?;

            let description = old_entry
                .get("#")
                .and_then(Value::as_str)
                .unwrap_or("(no description)");

            println!("Renaming flag '{old}' ({description})");

            // Validate new flag doesn't exist
            if let Some(new_entry) = affixes_obj.get(&new).or_else(|| properties_obj.get(&new)) {
                let new_desc = new_entry
                    .get("#")
                    .and_then(Value::as_str)
                    .unwrap_or("(no description)");
                return Err(anyhow!(
                    "Cannot rename to '{new}': flag already exists and is used for: {new_desc}"
                ));
            }

            // Create backups
            let backup_dict = format!("{}.bak", dict_path.display());
            let backup_affixes = format!("{}.bak", affixes_path.display());
            fs::copy(&dict_path, &backup_dict)
                .map_err(|e| anyhow!("Failed to create dictionary backup: {e}"))?;
            fs::copy(&affixes_path, &backup_affixes)
                .map_err(|e| anyhow!("Failed to create affixes backup: {e}"))?;

            // Update dictionary with proper comment and whitespace handling
            let dict_content = fs::read_to_string(&dict_path)
                .map_err(|e| anyhow!("Failed to read dictionary: {e}"))?;

            let updated_dict = dict_content
                .lines()
                .map(|line| {
                    if line.is_empty() || line.starts_with('#') {
                        return line.to_string();
                    }

                    let hash_pos = line.find('#').unwrap_or(line.len());
                    let (entry_part, comment_part) = line.split_at(hash_pos);

                    let slash_pos = entry_part.find('/').unwrap_or(entry_part.len());
                    let (lexeme, annotation) = entry_part.split_at(slash_pos);

                    format!(
                        "{}{}{}",
                        lexeme,
                        annotation.replace(&old, &new),
                        comment_part
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            // Update affixes (text-based replacement with context awareness)
            let updated_affixes_string =
                affixes_string.replace(&format!("\"{}\":", &old), &format!("\"{}\":", &new));

            // Verify that the updated affixes string is valid JSON
            serde_json::from_str::<Value>(&updated_affixes_string)
                .map_err(|e| anyhow!("Failed to parse updated annotations.json: {e}"))?;

            // Write changes
            fs::write(&dict_path, updated_dict)
                .map_err(|e| anyhow!("Failed to write updated dictionary: {e}"))?;
            fs::write(&affixes_path, updated_affixes_string)
                .map_err(|e| anyhow!("Failed to write updated affixes: {e}"))?;

            println!("Successfully renamed flag '{old}' to '{new}'");
            println!("  Description: {description}");
            println!("  Backups created at:\n    {backup_dict}\n    {backup_affixes}");

            Ok(())
        }
        Args::AuditDictionary { dir } => {
            let annotations_path = dir.join("annotations.json");
            let annotations_content = fs::read_to_string(&annotations_path)
                .map_err(|e| anyhow!("Failed to read annotations: {e}"))?;
            let annotations_json: Value = serde_json::from_str(&annotations_content)
                .map_err(|e| anyhow!("Failed to parse annotations.json: {e}"))?;

            let annotations = annotations_json
                .as_object()
                .ok_or_else(|| anyhow!("annotations.json is not an object"))?;

            let (affixes, properties) = ["affixes", "properties"]
                .iter()
                .map(|key| {
                    annotations
                        .get(*key)
                        .and_then(Value::as_object)
                        .ok_or_else(|| {
                            anyhow!("Missing or invalid '{key}' key in annotations.json")
                        })
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|v| (v[0], v[1]))?;

            let all_keys = affixes.keys().chain(properties.keys()).collect::<Vec<_>>();

            let mut annotation_flag_count: HashMap<char, u32> = all_keys
                .iter()
                .filter_map(|key| key.chars().next()) // Get first char of each key
                .map(|c| (c, 0))
                .collect();

            // let mut duplicate_flag_total = 0;
            let mut duplicate_flags = std::collections::HashMap::new();
            let mut unknown_flags = std::collections::HashMap::new();
            let mut unused_flag_total = 0;

            let dict_path = dir.join("dictionary.dict");
            let dict_content = fs::read_to_string(&dict_path)
                .map_err(|e| anyhow!("Failed to read dictionary: {e}"))?;

            for (line_num, line) in dict_content.lines().enumerate() {
                if line.is_empty()
                    || line.starts_with('#')
                    || line.chars().all(|c| c.is_ascii_digit())
                {
                    continue;
                }

                let (entry_part, _comment_part) =
                    line.split_once('#').map_or((line, ""), |(e, c)| (e, c));

                if let Some((lexeme, rest)) = entry_part.split_once('/') {
                    let (annotation, _whitespace) = match rest.split_once([' ', '\t']) {
                        Some((a, _)) => (a, &rest[a.len()..]),
                        None => (rest, ""),
                    };

                    let mut seen_flags = hashbrown::HashSet::new();

                    for flag in annotation.chars() {
                        if !seen_flags.insert(flag) {
                            eprintln!(
                                "Warning: Line {}: Duplicate annotation flag '{}' in entry: {}/{}",
                                line_num + 1,
                                flag,
                                lexeme,
                                annotation
                            );
                            // duplicate_flag_total += 1;
                            *duplicate_flags.entry(flag).or_insert(0) += 1;
                        }
                        if !annotation_flag_count.contains_key(&flag) {
                            eprintln!(
                                "Warning: Line {}: Unknown annotation flag '{}' in entry: {}/{}",
                                line_num + 1,
                                flag,
                                lexeme,
                                annotation
                            );
                            *unknown_flags.entry(flag).or_insert(0) += 1;
                        } else {
                            *annotation_flag_count.get_mut(&flag).unwrap() += 1;
                        }
                    }
                }
            }

            for (flag, count) in annotation_flag_count {
                if count == 0 {
                    eprintln!("Warning: Unused annotation flag '{}'", flag);
                    unused_flag_total += 1;
                }
            }

            let duplicate_flag_total = duplicate_flags.values().sum::<usize>();
            let unknown_flag_total = unknown_flags.values().sum::<usize>();

            if duplicate_flag_total > 0 || unknown_flag_total > 0 || unused_flag_total > 0 {
                eprintln!("\nAudit found issues:");
                if duplicate_flag_total > 0 {
                    eprintln!(
                        "  - {} duplicate flags found in {} entries",
                        duplicate_flags.len(),
                        duplicate_flag_total
                    );
                }
                if !unknown_flags.is_empty() {
                    let total_unknown = unknown_flags.values().sum::<usize>();
                    eprintln!(
                        "  - {} unknown flags found in {} entries",
                        unknown_flags.len(),
                        total_unknown
                    );
                }
                if unused_flag_total > 0 {
                    eprintln!("  - {} unused flags found", unused_flag_total);
                }
                std::process::exit(1);
            }

            Ok(())
        }
        Args::Compounds => {
            let mut compound_map: HashMap<String, Vec<String>> = HashMap::new();

            // First pass: process open and hyphenated compounds
            for word in curated_dictionary.words_iter() {
                if !word.contains(&' ') && !word.contains(&'-') {
                    continue;
                }

                let normalized_key: String = word
                    .iter()
                    .filter(|&&c| c != ' ' && c != '-')
                    .collect::<String>()
                    .to_lowercase();

                let word_str = word.iter().collect::<String>();
                compound_map
                    .entry(normalized_key)
                    .or_default()
                    .push(word_str);
            }

            // Second pass: process closed compounds
            for word in curated_dictionary.words_iter() {
                if word.contains(&' ') || word.contains(&'-') {
                    continue;
                }

                let normalized_key: String = word.iter().collect::<String>().to_lowercase();
                if let Some(variants) = compound_map.get_mut(&normalized_key) {
                    variants.push(word.iter().collect());
                }
            }

            // Process and print results
            let mut results: Vec<_> = compound_map
                .into_iter()
                .filter(|(_, v)| v.len() > 1)
                .collect();
            results.sort_by_key(|(k, _)| k.clone());

            // Instead of moving `results` into the for loop, iterate over a reference to it
            for (normalized, originals) in &results {
                println!("\nVariants for '{normalized}':");
                for original in originals {
                    println!("  - {original}");
                }
            }

            println!("\nFound {} compound word groups", results.len());
            Ok(())
        }
        Args::CaseVariants => {
            let case_bitmask = OrthFlags::LOWERCASE
                | OrthFlags::TITLECASE
                | OrthFlags::ALLCAPS
                | OrthFlags::LOWER_CAMEL
                | OrthFlags::UPPER_CAMEL;
            let mut processed_words = HashMap::new();
            let mut longest_word = 0;
            for word in curated_dictionary.words_iter() {
                if let Some(metadata) = curated_dictionary.get_word_metadata(word) {
                    let orth = metadata.orth_info;
                    let bits = orth.bits() & case_bitmask.bits();

                    if bits.count_ones() > 1 {
                        longest_word = longest_word.max(word.len());
                        // Mask out all bits except the case-related ones before printing
                        processed_words.insert(
                            word.to_string(),
                            OrthFlags::from_bits_truncate(orth.bits() & case_bitmask.bits()),
                        );
                    }
                }
            }
            let mut processed_words: Vec<_> = processed_words.into_iter().collect();
            processed_words.sort_by_key(|(word, _)| word.clone());
            let longest_num = (processed_words.len() - 1).to_string().len();
            for (i, (word, orth)) in processed_words.iter().enumerate() {
                println!("{i:>longest_num$} {word:<longest_word$} : {orth:?}");
            }
            Ok(())
        }
        Args::NominalPhrases { input } => {
            // Get input from either file or direct text
            let (doc, _) = input
                .unwrap_or_read_from_stdin()
                .load(MarkdownOptions::default(), &curated_dictionary)?;

            let phrases: Vec<_> = doc
                .iter_nominal_phrases()
                .map(|toks| {
                    (
                        toks.first().unwrap().span.start,
                        toks.last().unwrap().span.end,
                    )
                })
                .collect();

            let mut last_end = 0;

            for (start, end) in phrases {
                // Plain text between nominal phrases
                if start > last_end {
                    let span = Span::new(last_end, start);
                    let txt = doc.get_span_content_str(&span);
                    if !txt.trim().is_empty() {
                        print!("{}", txt);
                    }
                }

                // Highlighted nominal phrase
                let span = Span::new(start, end);
                let txt = doc.get_span_content_str(&span);

                if color {
                    print!("\x1b[33m{}\x1b[0m", txt);
                } else {
                    print!("{}", txt);
                }

                last_end = end;
            }

            // Plain text after the last nominal phrase, if any
            let doc_len = doc.get_full_content().len();
            if last_end < doc_len {
                let span = Span::new(last_end, doc_len);
                let txt = doc.get_span_content_str(&span);
                if !txt.trim().is_empty() {
                    print!("{}", txt);
                }
            }

            println!();

            Ok(())
        }
        Args::Test { input } => {
            let weir_file = fs::read_to_string(input)?;
            let mut linter = WeirLinter::new(&weir_file)?;

            let failing_tests = linter.run_tests();

            if failing_tests.is_empty() {
                eprintln!("All tests pass!");
                Ok(())
            } else {
                eprintln!("{:?}", failing_tests);
                process::exit(1);
            }
        }
    }
}

/// Parse a dialect string into a Dialect enum value.
/// Supports common synonyms, abbreviations, and codes.
fn parse_dialect(dialect: &str) -> anyhow::Result<Dialect> {
    match dialect.to_lowercase().as_str() {
        "us" | "usa" | "america" | "american" | "en-us" | "en_us" => Ok(Dialect::American),
        "uk" | "gb" | "british" | "britain" | "en-gb" | "en_gb" => Ok(Dialect::British),
        "au" | "aus" | "australia" | "australian" | "en-au" | "en_au" => Ok(Dialect::Australian),
        "in" | "india" | "indian" | "bharat" | "en-in" | "en_in" => Ok(Dialect::Indian),
        "ca" | "canada" | "canadian" | "en-ca" | "en_ca" => Ok(Dialect::Canadian),
        _ => Err(anyhow!("Unknown dialect: {}", dialect)),
    }
}

/// Split a dictionary line into its word and annotation segments
fn line_to_parts(line: &str) -> (String, String) {
    if let Some((word, annot)) = line.split_once('/') {
        (word.to_owned(), annot.to_string())
    } else {
        (line.to_owned(), String::new())
    }
}

fn print_word_derivations(word: &str, annot: &str, dictionary: &impl Dictionary) {
    println!("{word}/{annot}");

    let id = WordId::from_word_str(word);

    let children = dictionary
        .words_iter()
        .filter(|e| dictionary.get_word_metadata(e).unwrap().derived_from == Some(id));

    println!(" - {word}");

    for child in children {
        let child_str: String = child.iter().collect();
        println!(" - {child_str}");
    }
}

#![doc = include_str!("../README.md")]

use either::*;
use harper_core::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary, WordId};
use hashbrown::HashMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use anyhow::anyhow;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use clap::Parser;
use dirs::{config_dir, data_local_dir};
use harper_comments::CommentParser;
use harper_core::linting::{Lint, LintGroup, LintKind};
use harper_core::parsers::{Markdown, MarkdownOptions, OrgMode, PlainEnglish};
use harper_core::{
    CharStringExt, Dialect, DictWordMetadata, Document, Span, Token, TokenKind, TokenStringExt,
    dict_word_metadata_orthography::OrthFlags, remove_overlaps_map,
};
use harper_ink::InkParser;
use harper_literate_haskell::LiterateHaskellParser;
#[cfg(feature = "training")]
use harper_pos_utils::{BrillChunker, BrillTagger, BurnChunkerCpu};
use harper_python::PythonParser;

use harper_stats::Stats;
use serde::Serialize;
use serde_json::Value;

mod input;
use input::Input;

mod annotate_tokens;
use annotate_tokens::{Annotation, AnnotationType};

/// A debugging tool for the Harper grammar checker.
#[derive(Debug, Parser)]
#[command(version, about)]
enum Args {
    /// Lint provided documents.
    Lint {
        /// The text or file you wish to grammar check. If not provided, it will be read from
        /// standard input.
        inputs: Vec<Input>,
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
        /// Specify the dialect.
        #[arg(short, long, default_value = Dialect::American.to_string())]
        dialect: Dialect,
        /// Path to the user dictionary.
        #[arg(short, long, default_value = config_dir().unwrap().join("harper-ls/dictionary.txt").into_os_string())]
        user_dict_path: PathBuf,
        /// Path to the directory for file-local dictionaries.
        #[arg(short, long, default_value = data_local_dir().unwrap().join("harper-ls/file_dictionaries/").into_os_string())]
        file_dict_path: PathBuf,
    },
    /// Parse a provided document and print the detected symbols.
    Parse {
        /// The text or file you wish to parse. If not provided, it will be read from standard
        /// input.
        input: Option<Input>,
    },
    /// Parse a provided document and show the spans of the detected tokens.
    Spans {
        /// The file or text for which you wish to display the spans. If not provided, it will be
        /// read from standard input.
        input: Option<Input>,
        /// Include newlines in the output
        #[arg(short, long)]
        include_newlines: bool,
    },
    /// Parse a provided document and annotate its tokens.
    AnnotateTokens {
        /// The text or file you wish to parse. If not provided, it will be read from standard
        /// input.
        input: Option<Input>,
        /// How the tokens should be annotated.
        #[arg(short, long, value_enum, default_value_t = AnnotationType::Upos)]
        annotation_type: AnnotationType,
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
        file: PathBuf,
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
        input: Option<Input>,
    },
}

struct LintOptions<'a> {
    count: bool,
    ignore: &'a Option<Vec<String>>,
    only: &'a Option<Vec<String>>,
    dialect: Dialect,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let markdown_options = MarkdownOptions::default();
    let curated_dictionary = FstDictionary::curated();

    match args {
        Args::Lint {
            inputs,
            count,
            ignore,
            only,
            dialect,
            user_dict_path,
            // TODO workspace_dict_path?
            file_dict_path,
        } => {
            lint(
                markdown_options,
                curated_dictionary,
                inputs,
                LintOptions {
                    count,
                    ignore: &ignore,
                    only: &only,
                    dialect,
                },
                user_dict_path,
                // TODO workspace_dict_path?
                file_dict_path,
            )
        }
        Args::Parse { input } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_else(|| Input::try_from_stdin().unwrap());

            // Load the file/text.
            let (doc, _) = input.load(false, markdown_options, &curated_dictionary)?;
            let doc = doc.expect("Failed to load document");

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
            let input = input.unwrap_or_else(|| Input::try_from_stdin().unwrap());

            // Load the file/text.
            let (doc, source) = input.load(false, markdown_options, &curated_dictionary)?;
            let doc = doc.expect("Failed to load document");

            let primary_color = Color::Blue;
            let secondary_color = Color::Magenta;
            let unlintable_color = Color::Red;
            let input_identifier = input.get_identifier();

            let mut report_builder = Report::build(
                ReportKind::Custom("Spans", primary_color),
                &input_identifier,
                0,
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
        Args::AnnotateTokens {
            input,
            annotation_type,
        } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_else(|| Input::try_from_stdin().unwrap());

            // Load the file/text.
            let (doc, source) = input.load(false, markdown_options, &curated_dictionary)?;
            let doc = doc.expect("Failed to load document");

            let input_identifier = input.get_identifier();

            let mut report_builder = Report::build(
                ReportKind::Custom("AnnotateTokens", Color::Blue),
                &*input_identifier,
                0,
            );

            report_builder = report_builder.with_labels(Annotation::iter_labels_from_document(
                annotation_type,
                &doc,
                &input_identifier,
            ));

            let report = report_builder.finish();
            report.print((&*input_identifier, Source::from(source)))?;

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
        Args::MineWords { file } => {
            let (doc, _source) = load_file(
                &file,
                None,
                false,
                MarkdownOptions::default(),
                &curated_dictionary,
            )?;
            let doc = doc.expect("Failed to load document");

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
            let input = match input {
                Some(Input::File(path)) => std::fs::read_to_string(path)?,
                Some(Input::Dir(_)) => anyhow::bail!("Directory input is not supported"),
                Some(Input::Text(text)) | Some(Input::Stdin(text)) => text,
                None => std::io::read_to_string(std::io::stdin())?,
            };

            let doc = Document::new_markdown_default_curated(&input);
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

                print!("\x1b[33m{}\x1b[0m", txt);

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
    }
}

fn load_file(
    file: &Path,
    input_identifier: Option<&str>,
    batch_mode: bool,
    markdown_options: MarkdownOptions,
    dictionary: &impl Dictionary,
) -> anyhow::Result<(Option<Document>, String)> {
    let source = std::fs::read_to_string(file)?;

    let parser: Box<dyn harper_core::parsers::Parser> = match file
        .extension()
        .map(|v| v.to_str().unwrap())
    {
        Some("md") => Box::new(Markdown::default()),
        Some("ink") => Box::new(InkParser::default()),

        Some("lhs") => Box::new(LiterateHaskellParser::new_markdown(
            MarkdownOptions::default(),
        )),
        Some("org") => Box::new(OrgMode),
        Some("typ") => Box::new(harper_typst::Typst),
        Some("py") | Some("pyi") => Box::new(PythonParser::default()),
        Some("txt") => Box::new(PlainEnglish),
        _ => {
            if let Some(comment_parser) = CommentParser::new_from_filename(file, markdown_options) {
                Box::new(comment_parser)
            } else {
                eprintln!(
                    "{}Warning: Could not detect language ID; {}",
                    input_identifier
                        .map(|id| format!("{}: ", id))
                        .unwrap_or_default(),
                    if batch_mode {
                        "skipping file."
                    } else {
                        "falling back to PlainEnglish parser."
                    }
                );
                if batch_mode {
                    return Ok((None, source));
                } else {
                    Box::new(PlainEnglish)
                }
            }
        }
    };

    Ok((Some(Document::new(&source, &parser, dictionary)), source))
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

enum ReportStyle {
    FullAriadneLintReport,
    BriefCountsOnlyLintReport,
}

fn lint(
    markdown_options: MarkdownOptions,
    curated_dictionary: Arc<dyn Dictionary>,
    inputs: Vec<Input>,
    lint_options: LintOptions,
    user_dict_path: PathBuf,
    // TODO workspace_dict_path?
    file_dict_path: PathBuf,
) -> anyhow::Result<()> {
    let LintOptions {
        count,
        ignore,
        only,
        dialect,
    } = lint_options;

    // Zero or more inputs, default to stdin if not provided
    let all_user_inputs = if inputs.is_empty() {
        vec![Input::try_from_stdin().unwrap()]
    } else {
        inputs
    };

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

    for user_input in all_user_inputs {
        let (batch_mode, maybe_dir) = match &user_input {
            Input::Dir(dir) => (true, std::fs::read_dir(dir).ok()),
            _ => (false, None),
        };

        // All the files within this input if it's a Dir, or just this input otherwise.
        let inputs = if let Some(dir) = maybe_dir {
            Either::Left(
                dir.filter_map(Result::ok)
                    .filter(|entry| entry.file_type().map(|ft| !ft.is_dir()).unwrap_or(false))
                    .map(|entry| Input::File(entry.path())),
            )
        } else {
            Either::Right(std::iter::once(user_input.clone()))
        };

        for current_input in inputs {
            let lint_results = lint_one_input(
                // Common properties of harper-cli
                markdown_options,
                &curated_plus_user_dict,
                // Passed from the user for the `lint` subcommand
                &report_mode,
                LintOptions {
                    count,
                    ignore,
                    only,
                    dialect,
                },
                &file_dict_path,
                // Are we linting multiple inputs inside a directory?
                batch_mode,
                // The current input to be linted
                current_input,
            );

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
    }

    final_report(
        dialect,
        true,
        all_lint_kinds,
        all_rules,
        all_lint_kind_rule_pairs,
        all_spellos,
    );

    process::exit(1);
}

type LintKindCount = HashMap<LintKind, usize>;
type LintRuleCount = HashMap<String, usize>;
type LintKindRulePairCount = HashMap<(LintKind, String), usize>;
type SpelloCount = HashMap<String, usize>;

struct InputInfo {
    input: Input,
    doc: Document,
    source: String,
}

fn lint_one_input(
    // Common properties of harper-cli
    markdown_options: MarkdownOptions,
    curated_plus_user_dict: &MergedDictionary,
    report_mode: &ReportStyle,
    // Options passed from the user specific to the `lint` subcommand
    lint_options: LintOptions,
    file_dict_path: &Path,
    // Are we linting multiple inputs?
    batch_mode: bool,
    // For the current input
    current_input: Input,
) -> (
    LintKindCount,
    LintRuleCount,
    LintKindRulePairCount,
    SpelloCount,
) {
    let LintOptions {
        count: _,
        ignore,
        only,
        dialect,
    } = lint_options;

    let mut lint_kinds: HashMap<LintKind, usize> = HashMap::new();
    let mut lint_rules: HashMap<String, usize> = HashMap::new();
    let mut lint_kind_rule_pairs: HashMap<(LintKind, String), usize> = HashMap::new();
    let mut spellos: HashMap<String, usize> = HashMap::new();

    // Create a new merged dictionary for this input.
    let mut dictionary = curated_plus_user_dict.clone();

    // If processing a file, try to load its per-file dictionary
    if let Input::File(ref file) = current_input {
        let dict_path = file_dict_path.join(file_dict_name(file));
        if let Ok(file_dictionary) = load_dict(&dict_path) {
            dictionary.add_dictionary(Arc::new(file_dictionary));
            println!(
                "{}: Note: Using per-file dictionary: {}",
                &current_input.get_identifier(),
                dict_path.display()
            );
        }
    }

    match current_input.load(batch_mode, markdown_options, &dictionary) {
        Err(err) => eprintln!("{}", err),
        Ok((maybe_doc, source)) => {
            if let Some(doc) = maybe_doc {
                // Create the Lint Group from which we will lint this input, using the combined dictionary and the specified dialect
                let mut lint_group = LintGroup::new_curated(dictionary.into(), dialect);

                // Turn specified rules on or off
                configure_lint_group(&mut lint_group, only, ignore);

                // Run the linter, getting back a map of rule name -> lints
                let mut named_lints = lint_group.organized_lints(&doc);

                // Lint counts, for brief reporting
                let lint_count_before = named_lints.values().map(|v| v.len()).sum::<usize>();
                remove_overlaps_map(&mut named_lints);
                let lint_count_after = named_lints.values().map(|v| v.len()).sum::<usize>();

                // Extract the lint kinds and rules etc. for reporting
                (lint_kinds, lint_rules) = count_lint_kinds_and_rules(&named_lints);
                lint_kind_rule_pairs = collect_lint_kind_rule_pairs(&named_lints);
                spellos = collect_spellos(&named_lints, doc.get_source());

                single_input_report(
                    &InputInfo {
                        input: current_input,
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

    (lint_kinds, lint_rules, lint_kind_rule_pairs, spellos)
}

fn configure_lint_group(
    lint_group: &mut LintGroup,
    only: &Option<Vec<String>>,
    ignore: &Option<Vec<String>>,
) {
    if let Some(rules) = only {
        lint_group.set_all_rules_to(Some(false));

        for rule in rules {
            if !lint_group.contains_key(rule) {
                eprintln!("Warning: Cannot enable unknown rule '{}'.", &rule);
            }
            lint_group.config.set_rule_enabled(rule, true);
        }
    }

    if let Some(rules) = ignore {
        for rule in rules {
            if !lint_group.contains_key(rule) {
                eprintln!("Warning: Cannot disable unknown rule '{}'.", &rule);
            }
            lint_group.config.set_rule_enabled(rule, false);
        }
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
    input_info: &InputInfo,
    // Linting results of this input
    named_lints: &BTreeMap<String, Vec<Lint>>,
    lint_count: (usize, usize),
    lint_kinds: &HashMap<LintKind, usize>,
    lint_rules: &HashMap<String, usize>,
    // Reporting parameters
    batch_mode: bool, // If true, we are processing multiple files, which affects how we report
    report_mode: &ReportStyle,
) {
    let InputInfo {
        input: current_input,
        doc,
        source,
    } = input_info;
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
            &current_input.get_identifier()
        );
    }

    // Report the number of lints no matter what report mode we are in
    println!(
        "{}: {}",
        current_input.get_identifier(),
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

        let input_identifier = current_input.get_identifier();

        if lint_count_after != 0 {
            let mut report_builder = Report::build(ReportKind::Advice, &input_identifier, 0);

            for (rule_name, lints) in named_lints {
                for lint in lints {
                    let (r, g, b) = rgb_for_lint_kind(Some(&lint.lint_kind));
                    report_builder = report_builder.with_label(
                        Label::new((&input_identifier, lint.span.into()))
                            .with_message(format!(
                                "{}: {}",
                                format_args!("[{}::{}]", lint.lint_kind, rule_name)
                                    .fg(ariadne::Color::Rgb(r, g, b)),
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
        print_formatted_items(lk_vec);
    }

    if !lint_rules.is_empty() {
        let mut rules_vec: Vec<_> = lint_rules.iter().collect();
        rules_vec.sort_by_key(|(rn, count)| (std::cmp::Reverse(**count), rn.to_string()));

        let r_vec: Vec<(Option<String>, String)> = rules_vec
            .into_iter()
            .map(|(rn, c)| (None, format!("<{rn}: {c}>")))
            .collect();

        println!("rules:");
        print_formatted_items(r_vec);
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
            print_formatted_items(lint_kind_counts);
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
            print_formatted_items(rule_name_counts);
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
        print_formatted_items(formatted_lint_kind_rule_pairs);
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
                let color = format!("\x1b[38;2;{};{};{}m", r, g, b);

                variants
                    .into_iter()
                    .map(move |(spelling, c)| (Some(color.clone()), format!("(“{spelling}”: {c})")))
            })
            .collect();

        println!("All files Spelling::SpellCheck (For dialect: {})", dialect);
        print_formatted_items(spelling_vec);
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

fn print_formatted_items(items: impl IntoIterator<Item = (Option<String>, String)>) {
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

        let (set, reset): (&str, &str) = if let Some(prefix) = ansi.as_ref() {
            (prefix, "\x1b[0m")
        } else {
            ("", "")
        };
        print!("{}{}{}{}", before, set, text, reset);
        len_so_far += len_to_add;
        first_on_line = false;
    }
    println!();
}

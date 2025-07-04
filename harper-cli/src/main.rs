#![doc = include_str!("../README.md")]

use hashbrown::HashMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use anyhow::anyhow;
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use dirs::{config_dir, data_local_dir};
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Markdown, MarkdownOptions, OrgMode, PlainEnglish};
use harper_core::{
    CharStringExt, Dialect, Dictionary, Document, FstDictionary, MergedDictionary,
    MutableDictionary, TokenKind, TokenStringExt, WordId, WordMetadata, remove_overlaps,
};
use harper_literate_haskell::LiterateHaskellParser;
use harper_pos_utils::{BrillChunker, BrillTagger};
use harper_stats::Stats;
use serde::Serialize;

mod input;
use input::Input;

/// A debugging tool for the Harper grammar checker.
#[derive(Debug, Parser)]
#[command(version, about)]
enum Args {
    /// Lint a provided document.
    Lint {
        /// The text or file you wish to grammar check. If not provided, it will be read from
        /// standard input.
        input: Option<Input>,
        /// Whether to merely print out the number of errors encountered,
        /// without further details.
        #[arg(short, long)]
        count: bool,
        /// Restrict linting to only a specific set of rules.
        /// If omitted, `harper-cli` will run every rule.
        #[arg(short, long, value_delimiter = ',')]
        only_lint_with: Option<Vec<String>>,
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
    /// Get the metadata associated with a particular word.
    Metadata { word: String },
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
    /// Emit a decompressed, line-separated list of the compounds in Harper's dictionary.
    /// As long as there's either an open or hyphenated spelling.
    Compounds,
    /// Provided a sentence or phrase, emit a list of each noun phrase contained within.
    NominalPhrases { input: String },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let markdown_options = MarkdownOptions::default();
    let dictionary = FstDictionary::curated();

    match args {
        Args::Lint {
            input,
            count,
            only_lint_with,
            dialect,
            user_dict_path,
            file_dict_path,
        } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_else(|| Input::try_from_stdin().unwrap());

            let mut merged_dict = MergedDictionary::new();
            merged_dict.add_dictionary(dictionary);

            // Attempt to load user dictionary.
            match load_dict(&user_dict_path) {
                Ok(user_dict) => merged_dict.add_dictionary(Arc::new(user_dict)),
                Err(err) => println!("{}: {}", user_dict_path.display(), err),
            }

            if let Input::File(ref file) = input {
                // Only attempt to load file dictionary if input is a file.
                let file_dict_path = file_dict_path.join(file_dict_name(file));
                match load_dict(&file_dict_path) {
                    Ok(file_dict) => merged_dict.add_dictionary(Arc::new(file_dict)),
                    Err(err) => println!("{}: {}", file_dict_path.display(), err),
                }
            }

            // Load the file/text.
            let (doc, source) = input.load(markdown_options, &merged_dict)?;

            let mut linter = LintGroup::new_curated(Arc::new(merged_dict), dialect);

            if let Some(rules) = only_lint_with {
                linter.set_all_rules_to(Some(false));

                for rule in rules {
                    linter.config.set_rule_enabled(rule, true);
                }
            }

            let mut lints = linter.lint(&doc);

            if count {
                println!("{}", lints.len());
                return Ok(());
            }

            if lints.is_empty() {
                println!("No lints found");
                return Ok(());
            }

            remove_overlaps(&mut lints);

            let primary_color = Color::Magenta;

            let input_identifier = input.get_identifier();

            let mut report_builder = Report::build(ReportKind::Advice, &input_identifier, 0);

            for lint in lints {
                report_builder = report_builder.with_label(
                    Label::new((&input_identifier, lint.span.into()))
                        .with_message(lint.message)
                        .with_color(primary_color),
                );
            }

            let report = report_builder.finish();
            report.print((&input_identifier, Source::from(source)))?;

            process::exit(1)
        }
        Args::Parse { input } => {
            // Try to read from standard input if `input` was not provided.
            let input = input.unwrap_or_else(|| Input::try_from_stdin().unwrap());

            // Load the file/text.
            let (doc, _) = input.load(markdown_options, &dictionary)?;

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
            let (doc, source) = input.load(markdown_options, &dictionary)?;

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
        Args::Words => {
            let mut word_str = String::new();

            for word in dictionary.words_iter() {
                word_str.clear();
                word_str.extend(word);

                println!("{word_str:?}");
            }

            Ok(())
        }
        Args::Metadata { word } => {
            let metadata = dictionary.get_word_metadata_str(&word);
            let json = serde_json::to_string_pretty(&metadata).unwrap();

            println!("{json}");

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

            let linter = LintGroup::new_curated(dictionary, Dialect::American);

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
            let (doc, _source) = load_file(&file, MarkdownOptions::default(), &dictionary)?;

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
        Args::RenameFlag { old, new, dir } => {
            use serde_json::Value;

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
        Args::Compounds => {
            let mut compound_map: HashMap<String, Vec<String>> = HashMap::new();

            // First pass: process open and hyphenated compounds
            for word in dictionary.words_iter() {
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
            for word in dictionary.words_iter() {
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
        Args::NominalPhrases { input } => {
            let doc = Document::new_markdown_default_curated(&input);

            for phrase in doc.iter_nominal_phrases() {
                let s =
                    doc.get_span_content_str(&phrase.span().ok_or(anyhow!("Unable to get span"))?);

                println!("{s}");
            }

            Ok(())
        }
    }
}

fn load_file(
    file: &Path,
    markdown_options: MarkdownOptions,
    dictionary: &impl Dictionary,
) -> anyhow::Result<(Document, String)> {
    let source = std::fs::read_to_string(file)?;

    let parser: Box<dyn harper_core::parsers::Parser> = match file
        .extension()
        .map(|v| v.to_str().unwrap())
    {
        Some("md") => Box::new(Markdown::default()),

        Some("lhs") => Box::new(LiterateHaskellParser::new_markdown(
            MarkdownOptions::default(),
        )),
        Some("org") => Box::new(OrgMode),
        Some("typ") => Box::new(harper_typst::Typst),
        _ => {
            if let Some(comment_parser) = CommentParser::new_from_filename(file, markdown_options) {
                Box::new(comment_parser)
            } else {
                println!(
                    "Warning: could not detect language ID; falling back to PlainEnglish parser."
                );
                Box::new(PlainEnglish)
            }
        }
    };

    Ok((Document::new(&source, &parser, dictionary), source))
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
            .map(|l| (l.chars().collect::<Vec<_>>(), WordMetadata::default())),
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

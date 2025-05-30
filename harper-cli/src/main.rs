#![doc = include_str!("../README.md")]

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use dirs::{config_dir, data_local_dir};
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Markdown, MarkdownOptions, PlainEnglish};
use harper_core::{
    remove_overlaps, CharStringExt, Dialect, Dictionary, Document, FstDictionary, MergedDictionary,
    MutableDictionary, TokenKind, TokenStringExt, WordId, WordMetadata,
};
use harper_literate_haskell::LiterateHaskellParser;
use harper_stats::Stats;
use serde::Serialize;

/// A debugging tool for the Harper grammar checker.
#[derive(Debug, Parser)]
#[command(version, about)]
enum Args {
    /// Lint a provided document.
    Lint {
        /// The file you wish to grammar check.
        file: PathBuf,
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
        /// The file you wish to parse.
        file: PathBuf,
    },
    /// Parse a provided document and show the spans of the detected tokens.
    Spans {
        /// The file you wish to display the spans.
        file: PathBuf,
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
    /// Print harper-core version.
    CoreVersion,
    /// Rename a flag in the dictionary and affixes.
    RenameFlag {
        /// The old flag.
        old_flag: String,
        /// The new flag.
        new_flag: String,
        /// The directory containing the dictionary and affixes.
        dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let markdown_options = MarkdownOptions::default();
    let dictionary = FstDictionary::curated();

    match args {
        Args::Lint {
            file,
            count,
            only_lint_with,
            dialect,
            user_dict_path,
            file_dict_path,
        } => {
            let mut merged_dict = MergedDictionary::new();
            merged_dict.add_dictionary(dictionary);

            match load_dict(&user_dict_path) {
                Ok(user_dict) => merged_dict.add_dictionary(Arc::new(user_dict)),
                Err(err) => println!("{}: {}", user_dict_path.display(), err),
            }

            let file_dict_path = file_dict_path.join(file_dict_name(&file));
            match load_dict(&file_dict_path) {
                Ok(file_dict) => merged_dict.add_dictionary(Arc::new(file_dict)),
                Err(err) => println!("{}: {}", file_dict_path.display(), err),
            }

            let (doc, source) = load_file(&file, markdown_options, &merged_dict)?;

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

            let filename = file
                .file_name()
                .map(|s| s.to_string_lossy().into())
                .unwrap_or("<file>".to_string());

            let mut report_builder = Report::build(ReportKind::Advice, &filename, 0);

            for lint in lints {
                report_builder = report_builder.with_label(
                    Label::new((&filename, lint.span.into()))
                        .with_message(lint.message)
                        .with_color(primary_color),
                );
            }

            let report = report_builder.finish();
            report.print((&filename, Source::from(source)))?;

            process::exit(1)
        }
        Args::Parse { file } => {
            let (doc, _) = load_file(&file, markdown_options, &dictionary)?;

            for token in doc.tokens() {
                let json = serde_json::to_string(&token)?;
                println!("{}", json);
            }

            Ok(())
        }
        Args::Spans {
            file,
            include_newlines,
        } => {
            let (doc, source) = load_file(&file, markdown_options, &dictionary)?;

            let primary_color = Color::Blue;
            let secondary_color = Color::Magenta;
            let unlintable_color = Color::Red;
            let filename = file
                .file_name()
                .map(|s| s.to_string_lossy().into())
                .unwrap_or("<file>".to_string());

            let mut report_builder =
                Report::build(ReportKind::Custom("Spans", primary_color), &filename, 0);
            let mut color = primary_color;

            for token in doc.tokens().filter(|t| {
                include_newlines
                    || !matches!(t.kind, TokenKind::Newline(_) | TokenKind::ParagraphBreak)
            }) {
                report_builder = report_builder.with_label(
                    Label::new((&filename, token.span.into()))
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
            report.print((&filename, Source::from(source)))?;

            Ok(())
        }
        Args::Words => {
            let mut word_str = String::new();

            for word in dictionary.words_iter() {
                word_str.clear();
                word_str.extend(word);

                println!("{:?}", word_str);
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
                        format!(
                            "'{}' is already in the dictionary but not annotated.",
                            dict_word
                        )
                    } else {
                        format!(
                            "'{}' is already in the dictionary with annotation `{}`.",
                            dict_word, dict_annot
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
                None => format!("'{}' is not in the dictionary yet.", word),
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
                    include_str!("../../harper-core/affixes.json"),
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
        Args::RenameFlag {
            old_flag,
            new_flag,
            dir,
        } => {
            use serde_json::Value;

            let dict_path = dir.join("dictionary.dict");
            let affixes_path = dir.join("affixes.json");

            // 1. Validate new flag is exactly one Unicode code point (Rust char)
            if new_flag.chars().count() != 1 {
                return Err(anyhow::anyhow!(
                    "New flag must be exactly one Unicode code point, got '{}' ({} chars)",
                    new_flag,
                    new_flag.chars().count()
                ));
            }

            // 2. Load and parse affixes
            let affixes_content = fs::read_to_string(&affixes_path)?;
            let mut affixes_json: Value = serde_json::from_str(&affixes_content)
                .map_err(|e| anyhow::anyhow!("Failed to parse affixes.json: {}", e))?;

            // Get the nested "affixes" object
            let affixes_obj = affixes_json.get_mut("affixes")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| anyhow::anyhow!("affixes.json does not contain 'affixes' object"))?;

            // 3. Validate old flag exists and get its description
            let old_entry = affixes_obj.get(&old_flag)
                .ok_or_else(|| anyhow::anyhow!("Flag '{}' not found in affixes.json", old_flag))?;

            let description = old_entry.get("#")
                .and_then(Value::as_str)
                .unwrap_or("(no description)");

            println!("Renaming flag '{}' ({}): {}", old_flag, description, old_entry);

            // 4. Validate new flag doesn't exist
            if affixes_obj.contains_key(&new_flag) {
                let new_desc = affixes_obj.get(&new_flag)
                    .and_then(|v| v.get("#"))
                    .and_then(Value::as_str)
                    .unwrap_or("(no description)");

                return Err(anyhow::anyhow!(
                    "Cannot rename to '{}': flag already exists and is used for: {}", 
                    new_flag,
                    new_desc
                ));
            }

            // 5. Create backups
            let backup_dict = format!("{}.bak", dict_path.display());
            let backup_affixes = format!("{}.bak", affixes_path.display());
            fs::copy(&dict_path, &backup_dict)?;
            fs::copy(&affixes_path, &backup_affixes)?;

            // 6. Update dictionary with proper comment and whitespace handling
            let dict_content = fs::read_to_string(&dict_path)?;
            let updated_dict = dict_content
                .lines()
                .map(|line| {
                    // Skip empty lines and full-line comments
                    if line.is_empty() || line.starts_with('#') {
                        return line.to_string();
                    }

                    // Split into entry and comment, preserving whitespace
                    let (entry, comment) = if let Some(comment_start) = line.find('#') {
                        // Split at the exact position to preserve whitespace
                        let (e, c) = line.split_at(comment_start);
                        (e.trim_end(), c)
                    } else {
                        (line.trim_end(), "")
                    };

                    // Split word and attributes
                    if let Some((word_part, attr_part)) = entry.split_once('/') {
                        let updated_attr = attr_part.replace(&old_flag, &new_flag);
                        format!("{}/{}{}", word_part.trim_end(), updated_attr, comment)
                    } else if !entry.is_empty() {
                        // Preserve lines without attributes
                        format!("{}{}", entry, comment)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            // 7. Update affixes (text-based replacement with context awareness)
            let updated_affixes = affixes_content
                // Handle keys in object: "A": {
                .replace(&format!("\"{}\":", &old_flag), &format!("\"{}\":", &new_flag))
                // Handle array elements: ["A", "B"]
                .replace(&format!("\"{}\"", &old_flag), &format!("\"{}\"", &new_flag))
                // Handle standalone flags in arrays: [A, B]
                .replace(&format!(" {}", &old_flag), &format!(" {}", &new_flag))
                .replace(&format!(",{}", &old_flag), &format!(",{}", &new_flag));

            // 8. Write changes
            fs::write(&dict_path, updated_dict)?;
            fs::write(&affixes_path, updated_affixes)?;

            println!("Successfully renamed flag '{}' to '{}'", old_flag, new_flag);
            println!("  Description: {}", description);
            println!("  Backups created at:\n    {}\n    {}", backup_dict, backup_affixes);
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

    println!(" - {}", word);

    for child in children {
        let child_str: String = child.iter().collect();
        println!(" - {}", child_str);
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

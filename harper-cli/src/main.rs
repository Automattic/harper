#![doc = include_str!("../README.md")]

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use anyhow::format_err;
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use dirs::{config_dir, data_local_dir};
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Markdown, MarkdownOptions};
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
        #[arg(short, long)]
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
    /// Annotate a provided string.
    Annotate { text: String },
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
        Args::Annotate { text } => {
            // TODO: should be plain text? How?
            let parser = Box::new(Markdown::default());
            let doc = Document::new(&text, &parser, &dictionary);

            enum DictStatus {
                Exact,     // Word is in the dictionary as-is
                Lowercase, // Only lowercase version is in the dictionary
                Missing,   // Not in the dictionary at all
            }

            for (ci, chunk) in doc.iter_chunks().enumerate() {
                print!("chunk {}:", ci);

                for tok in chunk.iter() {
                    let toktext = doc.get_span_content(&tok.span);

                    let dict_status = if tok.kind.as_word().is_some() {
                        if dictionary.contains_exact_word(toktext) {
                            DictStatus::Exact
                        } else if dictionary.contains_exact_word(&toktext.to_lower()) {
                            DictStatus::Lowercase
                        } else {
                            DictStatus::Missing
                        }
                    } else {
                        DictStatus::Missing
                    };

                    let mut annot = String::new();
                    if tok.kind.is_whitespace() {
                        print!(" ");
                    } else if tok.kind.is_punctuation() {
                        print!("\x1b[2m{}\x1b[0m", toktext.iter().collect::<String>());
                    } else {
                        if tok.kind.is_noun() {
                            annot += "\x1b[31mn.\x1b[0m"; // red (31m)
                        }
                        if tok.kind.is_verb() {
                            annot += "\x1b[32mv.\x1b[0m"; // green (32m)
                        }
                        if tok.kind.is_adjective() {
                            annot += "\x1b[33madj.\x1b[0m"; // yellow (33m)
                        }
                        if tok.kind.is_adverb() {
                            annot += "\x1b[34madv.\x1b[0m"; // blue (34m)
                        }
                        if tok.kind.is_pronoun() {
                            annot += "\x1b[35mpr.\x1b[0m"; // magenta (35m)
                        }
                        if tok.kind.is_conjunction() {
                            annot += "\x1b[36mconj.\x1b[0m"; // cyan (36m)
                        }
                        if tok.kind.is_preposition() {
                            annot += "\x1b[91mprep.\x1b[0m"; // bright red (91m)
                        }
                        if tok.kind.is_determiner() {
                            annot += "\x1b[92mdet.\x1b[0m"; // bright green (92m)
                        }

                        // Underline if missing from dictionary
                        let underline = match dict_status {
                            DictStatus::Exact => "",
                            DictStatus::Lowercase => "3", // italic
                            DictStatus::Missing => "4",   // underline
                        };
                        print!(
                            "'\x1b[{}m{}\x1b[0m'",
                            underline,
                            toktext.iter().collect::<String>()
                        );
                        if !annot.is_empty() {
                            print!(".{annot}");
                        }
                    }
                }
                println!();
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

    let parser: Box<dyn harper_core::parsers::Parser> =
        match file.extension().map(|v| v.to_str().unwrap()) {
            Some("md") => Box::new(Markdown::default()),
            Some("lhs") => Box::new(LiterateHaskellParser::new_markdown(
                MarkdownOptions::default(),
            )),
            Some("typ") => Box::new(harper_typst::Typst),
            _ => Box::new(
                CommentParser::new_from_filename(file, markdown_options)
                    .map(Box::new)
                    .ok_or(format_err!("Could not detect language ID."))?,
            ),
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

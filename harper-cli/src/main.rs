#![doc = include_str!("../README.md")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::format_err;
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::parsers::{Markdown, MarkdownOptions};
use harper_core::spell::hunspell::parse_default_attribute_list;
use harper_core::spell::hunspell::word_list::parse_word_list;
use harper_core::{
    remove_overlaps, CharString, Dictionary, Document, FstDictionary, TokenKind, WordMetadata,
};
use harper_literate_haskell::LiterateHaskellParser;
use hashbrown::HashMap;
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
    Forms { words: Vec<String> },
    /// Emit a decompressed, line-separated list of the words in Harper's dictionary.
    Words,
    /// Print the default config with descriptions.
    Config,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let markdown_options = MarkdownOptions::default();
    let linting_options = LintGroupConfig::default();
    let dictionary = FstDictionary::curated();

    match args {
        Args::Lint { file, count } => {
            let (doc, source) = load_file(&file, markdown_options)?;

            let mut linter = LintGroup::new(linting_options, dictionary);
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

            Ok(())
        }
        Args::Parse { file } => {
            let (doc, _) = load_file(&file, markdown_options)?;

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
            let (doc, source) = load_file(&file, markdown_options)?;

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
        Args::Forms { words } => {
            let mut expanded: HashMap<CharString, WordMetadata> = HashMap::new();
            let attributes = parse_default_attribute_list();
            let total = words.len();

            for (index, word) in words.iter().enumerate() {
                expanded.clear();

                let hunspell_word_list = format!("1\n{word}");
                let words = parse_word_list(&hunspell_word_list.to_string()).unwrap();
                attributes.expand_marked_words(words, &mut expanded);

                println!(
                    "{}{}{}",
                    if index > 0 { "\n" } else { "" },
                    if total != 1 {
                        format!("{}/{}: ", index + 1, total)
                    } else {
                        "".to_string()
                    },
                    word
                );
                expanded.keys().for_each(|form| {
                    let string_form: String = form.iter().collect();
                    println!("  - {}", string_form);
                });
            }

            Ok(())
        }
        Args::Config => {
            #[derive(Serialize)]
            struct Config {
                default_value: bool,
                description: String,
            }

            let mut linter = LintGroup::new(linting_options, dictionary);
            linter.config.fill_default_values();

            let default_config: HashMap<String, bool> =
                serde_json::from_str(&serde_json::to_string(&linter.config).unwrap()).unwrap();

            // Use `BTreeMap` so output is sorted by keys.
            let mut configs = BTreeMap::new();
            for (key, desc) in linter.all_descriptions().to_vec_pairs() {
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
    }
}

fn load_file(file: &Path, markdown_options: MarkdownOptions) -> anyhow::Result<(Document, String)> {
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

    Ok((Document::new_curated(&source, &parser), source))
}

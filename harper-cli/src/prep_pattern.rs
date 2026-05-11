use anyhow::Result;

use harper_core::parsers::MarkdownOptions;
use harper_core::spell::FstDictionary;

use crate::input::single_input::SingleInputTrait;
use crate::input::{AnyInput, InputTrait};

/// GitHub issue: https://github.com/Automattic/harper/issues/3337
/// Find word + preposition patterns in text to help identify wrong prepositions
pub fn run_prep_pattern(
    markdown_options: MarkdownOptions,
    curated_dictionary: std::sync::Arc<FstDictionary>,
    inputs: Vec<AnyInput>,
    words: Vec<String>,
    preposition: String,
    format: crate::OutputFormat,
) -> Result<()> {
    // If no inputs provided, read from stdin
    let inputs = if inputs.is_empty() {
        vec![AnyInput::Single(
            crate::input::single_input::SingleInput::Stdin(crate::input::single_input::StdinInput),
        )]
    } else {
        inputs
    };

    // Process each input
    for input in inputs {
        if let Some(single_input) = input.try_as_single_ref() {
            let (_doc, source) =
                single_input.load(markdown_options, curated_dictionary.as_ref())?;

            // Simple string matching approach for now
            let mut found_matches = false;

            for (line_num, line) in source.lines().enumerate() {
                for word in &words {
                    let pattern = format!("{} {}", word, preposition);
                    if line.to_lowercase().contains(&pattern.to_lowercase()) {
                        found_matches = true;
                        match format {
                            crate::OutputFormat::Default => {
                                println!(
                                    "{}:{}: Found '{}' followed by '{}' - {}",
                                    input.get_identifier(),
                                    line_num + 1,
                                    word,
                                    preposition,
                                    line.trim()
                                );
                            }
                            crate::OutputFormat::Json => {
                                let match_data = serde_json::json!({
                                    "line": line_num + 1,
                                    "word": word,
                                    "preposition": preposition,
                                    "line_content": line.trim(),
                                    "pattern": pattern
                                });
                                println!("{}", serde_json::to_string(&match_data)?);
                            }
                            crate::OutputFormat::Compact => {
                                println!(
                                    "{}:{}: '{}' followed by '{}' - {}",
                                    input.get_identifier(),
                                    line_num + 1,
                                    word,
                                    preposition,
                                    line.trim()
                                );
                            }
                        }
                    }
                }
            }

            if !found_matches && matches!(format, crate::OutputFormat::Default) {
                eprintln!("No patterns found in {}", input.get_identifier());
            }
        }
    }

    Ok(())
}

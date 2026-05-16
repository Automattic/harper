use anyhow::Result;

use crate::input::{
    multi_input::MultiInputTrait,
    single_input::SingleInputTrait,
    {AnyInput, InputTrait},
};
use harper_core::{CharStringExt, Token, parsers::MarkdownOptions, spell::FstDictionary};
/// GitHub issue: https://github.com/Automattic/harper/issues/3337
/// Find word + preposition patterns in text to help identify wrong prepositions
pub fn run_prep_pattern(
    markdown_options: MarkdownOptions,
    curated_dictionary: std::sync::Arc<FstDictionary>,
    inputs: Vec<AnyInput>,
    words: Vec<String>,
    prepositions: Vec<String>,
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
        // Convert SingleInput to a one-element collection, MultiInput to its collection
        let single_inputs: Vec<_> = if let Some(single_input) = input.try_as_single_ref() {
            vec![single_input.clone()]
        } else if let Some(multi_input) = input.try_as_multi_ref() {
            multi_input.iter_inputs()?.collect()
        } else {
            continue;
        };

        let is_multi_file = single_inputs.len() > 1;
        let mut total_found = false;

        for single_input in single_inputs {
            let (doc, source) =
                match single_input.load(markdown_options, curated_dictionary.as_ref()) {
                    Ok(result) => result,
                    Err(e) if e.to_string().contains("stream did not contain valid UTF-8") => {
                        eprintln!(
                            "Warning: Skipping {} - invalid UTF-8 content",
                            single_input.get_identifier()
                        );
                        continue;
                    }
                    Err(e) => return Err(e),
                };

            let mut found_matches = false;

            // Use Harper's token system for proper word boundary matching
            let tokens: Vec<_> = doc.tokens().collect();
            let source_chars = source.chars().collect::<Vec<_>>();

            // Check if we should match any preposition or specific ones
            let match_any_preposition = prepositions.len() == 1 && prepositions[0].to_lowercase() == "any";
            let target_prepositions: Vec<String> = if match_any_preposition {
                Vec::new() // Will match any preposition
            } else {
                prepositions.iter().map(|p| p.to_lowercase()).collect()
            };

            for (window_idx, window) in tokens.windows(3).enumerate() {
                // Check for word + space + preposition pattern
                if let (Some(word_token), Some(space_token), Some(prep_token)) =
                    (window.get(0), window.get(1), window.get(2))
                {
                    if word_token.kind.is_word()
                        && space_token.kind.is_whitespace()
                        && prep_token.kind.is_preposition()
                    {
                        // Check if word matches any of our target words
                        let word_content = word_token.get_ch(&source_chars);
                        if word_content.eq_any_ignore_ascii_case_str(
                            &words.iter().map(|w| w.as_str()).collect::<Vec<_>>(),
                        ) {
                            // Check if preposition matches our target prepositions (or any)
                            let prep_str = prep_token.get_str(&source_chars).to_lowercase();
                            let preposition_matches = match_any_preposition || 
                                target_prepositions.iter().any(|target| prep_str.contains(target));
                            
                            if preposition_matches {
                                found_matches = true;
                                total_found = true;

                                // Find line number for this token once
                                let line_num = tokens[..window_idx]
                                    .iter()
                                    .filter(|t| t.kind.is_paragraph_break())
                                    .count();

                                match format {
                                    crate::OutputFormat::Default => {
                                        // Adapted from format_lint_match logic to properly handle tokens
                                        let fmt_tokens = |tokens: &[&Token]| {
                                            tokens
                                                .iter()
                                                .filter(|t| !t.kind.is_unlintable())
                                                .map(|t| t.get_str(&source_chars))
                                                .collect::<String>()
                                        };

                                        // Get context around the match (2 tokens before, 2 tokens after)
                                        let context_before =
                                            (window_idx as isize - 2).max(0) as usize;
                                        let context_after = (window_idx + 3 + 2).min(tokens.len());

                                        let before_tokens = &tokens[context_before..window_idx];
                                        let matched_tokens = &tokens[window_idx..=window_idx + 2]; // word + space + preposition
                                        let after_tokens = &tokens[window_idx + 3..context_after];

                                        println!(
                                            "{}:{}: \x1b[2m{}\x1b[0m\x1b[1;31m{}\x1b[0m\x1b[2m{}\x1b[0m",
                                            single_input.get_identifier(),
                                            line_num + 1,
                                            fmt_tokens(&before_tokens),
                                            fmt_tokens(&matched_tokens),
                                            fmt_tokens(&after_tokens)
                                        );
                                    }
                                    crate::OutputFormat::Json => {
                                        let match_data = serde_json::json!({
                                            "line": line_num + 1,
                                            "word": word_token.get_str(&source_chars),
                                            "preposition": prep_token.get_str(&source_chars),
                                            "word_span": (word_token.span.start, word_token.span.end),
                                            "prep_span": (prep_token.span.start, prep_token.span.end)
                                        });
                                        println!("{}", serde_json::to_string(&match_data)?);
                                    }
                                    crate::OutputFormat::Compact => {
                                        println!(
                                            "{}:{}: '{}' followed by '{}' - {}",
                                            single_input.get_identifier(),
                                            line_num + 1,
                                            word_token.get_str(&source_chars),
                                            prep_token.get_str(&source_chars),
                                            source[word_token.span.start..prep_token.span.end]
                                                .to_string()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Check for preposition + space + word pattern
                if let (Some(prep_token), Some(space_token), Some(word_token)) =
                    (window.get(0), window.get(1), window.get(2))
                {
                    if prep_token.kind.is_preposition()
                        && space_token.kind.is_whitespace()
                        && word_token.kind.is_word()
                    {
                        // Check if word matches any of our target words
                        let word_content = word_token.get_ch(&source_chars);
                        if word_content.eq_any_ignore_ascii_case_str(
                            &words.iter().map(|w| w.as_str()).collect::<Vec<_>>(),
                        ) {
                            // Check if preposition matches our target prepositions (or any)
                            let prep_str = prep_token.get_str(&source_chars).to_lowercase();
                            let preposition_matches = match_any_preposition || 
                                target_prepositions.iter().any(|target| prep_str.contains(target));
                            
                            if preposition_matches {
                                found_matches = true;
                                total_found = true;
                                
                                // Find line number for this token once
                                let line_num = tokens[..window_idx]
                                    .iter()
                                    .filter(|t| t.kind.is_paragraph_break())
                                    .count();
                                
                                match format {
                                    crate::OutputFormat::Default => {
                                        // Adapted from format_lint_match logic to properly handle tokens
                                        let fmt_tokens = |tokens: &[&Token]| {
                                            tokens
                                                .iter()
                                                .filter(|t| !t.kind.is_unlintable())
                                                .map(|t| t.get_str(&source_chars))
                                                .collect::<String>()
                                        };

                                        // Get context around the match (2 tokens before, 2 tokens after)
                                        let context_before = (window_idx as isize - 2).max(0) as usize;
                                        let context_after = (window_idx + 3 + 2).min(tokens.len());
                                        
                                        let before_tokens = &tokens[context_before..window_idx];
                                        let matched_tokens = &tokens[window_idx..=window_idx + 2]; // preposition + space + word
                                        let after_tokens = &tokens[window_idx + 3..context_after];
                                        
                                        println!(
                                            "{}:{}: \x1b[2m{}\x1b[0m\x1b[1;31m{}\x1b[0m\x1b[2m{}\x1b[0m",
                                            single_input.get_identifier(),
                                            line_num + 1,
                                            fmt_tokens(&before_tokens),
                                            fmt_tokens(&matched_tokens),
                                            fmt_tokens(&after_tokens)
                                        );
                                    }
                                    crate::OutputFormat::Json => {
                                        let match_data = serde_json::json!({
                                            "line": line_num + 1,
                                            "preposition": prep_token.get_str(&source_chars),
                                            "word": word_token.get_str(&source_chars),
                                            "prep_span": (prep_token.span.start, prep_token.span.end),
                                            "word_span": (word_token.span.start, word_token.span.end)
                                        });
                                        println!("{}", serde_json::to_string(&match_data)?);
                                    }
                                    crate::OutputFormat::Compact => {
                                        println!(
                                            "{}:{}: '{}' followed by '{}' - {}",
                                            single_input.get_identifier(),
                                            line_num + 1,
                                            prep_token.get_str(&source_chars),
                                            word_token.get_str(&source_chars),
                                            source[prep_token.span.start..word_token.span.end]
                                                .to_string()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Only show "No patterns found" for single files or when processing a single file
            if !found_matches && !is_multi_file && matches!(format, crate::OutputFormat::Default) {
                eprintln!("No patterns found in {}", single_input.get_identifier());
            }
        }

        // Show summary for multi-file processing
        if is_multi_file && !total_found && matches!(format, crate::OutputFormat::Default) {
            eprintln!("No patterns found in {}", input.get_identifier());
        }
    }

    Ok(())
}

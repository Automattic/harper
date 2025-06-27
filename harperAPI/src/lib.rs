use actix_web::{web, HttpResponse, Responder};
use harper_core::{
    linting::{Lint, LintGroup, LintKind, Linter},
    Dialect, Document, FstDictionary,
};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::cmp::Ordering;

/// This struct represents the nested "suggestions" object in the JSON output.
#[derive(Serialize)]
pub struct Suggestions {
    /// A list of suggested replacements for the linted text.
    pub recommendation: Vec<String>,
}

/// This struct represents a single lint suggestion in the desired JSON output format.
#[derive(Serialize)]
pub struct FormattedLintOutput {
    /// The starting byte index of the linted text, relative to the paragraph.
    pub start: usize,
    /// The length of the linted text in bytes.
    pub length: usize,
    /// The ending byte index of the linted text, relative to the paragraph.
    pub end: usize,
    /// A key for the paragraph, numbered sequentially starting from 1.
    #[serde(rename = "paragraphKey")]
    pub paragraph_key: String,
    pub paragraph: String,
    /// The text of the paragraph containing the linted text.
    #[serde(skip_serializing_if = "String::is_empty")]
    /// The actual text string that was linted.
    pub string: String,
    /// The category or type of the lint (e.g., "Spelling", "Grammar").
    #[serde(rename = "type")]
    pub r#type: String,
    /// A nested object containing the list of suggested replacements.
    pub suggestions: Suggestions,
}

/// This struct represents the incoming JSON request body.
#[derive(Deserialize)]
pub struct LintRequest {
    pub text: String,
}

/// Defines the priority of different lint types to resolve overlaps.
/// Higher variants have higher priority.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum LintKindPriority {
    Miscellaneous,
    Spelling,
    Style,
    Repetition,
    WordChoice,
}

/// Converts a `harper_core::linting::LintKind` to our priority enum.
fn get_priority(kind: &LintKind) -> LintKindPriority {
    // Correctly reference the enum variants from harper_core.
    // Based on compiler feedback, these are unit variants, not tuple variants.
    match kind {
        LintKind::WordChoice => LintKindPriority::WordChoice,
        LintKind::Repetition => LintKindPriority::Repetition,
        LintKind::Style => LintKindPriority::Style,
        LintKind::Spelling => LintKindPriority::Spelling,
        LintKind::Miscellaneous => LintKindPriority::Miscellaneous,
        // Add a catch-all for any other variants that might exist
        _ => LintKindPriority::Miscellaneous,
    }
}

/// This is the main handler for the `/lint` endpoint.
/// It takes a JSON object with a `text` field, runs the linter,
/// filters for overlapping suggestions, and returns a clean JSON array.
pub async fn lint_text(request: web::Json<LintRequest>) -> impl Responder {
    let document = Document::new_plain_english_curated(&request.text);
    let dictionary = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dictionary, Dialect::American);

    // Get all lints from the linter.
    let mut lints_from_linter = linter.lint(&document);

    // --- Overlap Filtering Logic ---

    // 1. Sort lints by priority in descending order (highest priority first).
    lints_from_linter.sort_by(|a, b| get_priority(&b.lint_kind).cmp(&get_priority(&a.lint_kind)));

    let mut final_lints: Vec<&Lint> = Vec::new();
    let mut claimed_spans: Vec<Range<usize>> = Vec::new();

    // 2. Iterate through sorted lints and filter overlaps.
    for lint in &lints_from_linter {
        // Correctly create a Range<usize> from the lint's Span
        let lint_range = lint.span.start..lint.span.end;

        let has_overlap = claimed_spans.iter().any(|claimed| {
            // Check for overlap: (start1 < end2) && (start2 < end1)
            lint_range.start < claimed.end && lint_range.end > claimed.start
        });

        // If there's no overlap, accept the lint and claim its span.
        if !has_overlap {
            final_lints.push(lint);
            // Push the created Range, not the Span itself.
            claimed_spans.push(lint_range);
        }
    }
    // --- End of Filtering Logic ---

    let para_offsets: Vec<(usize, usize)> = request
    // Generate paragraph boundaries.
        .text
        .split('\n')
        .scan(0, |offset, para_text| {
            let current_offset = *offset;
            *offset += para_text.len() + 1; // +1 for the newline
            // let current_para = para_text.to_string();
            Some(current_offset)
        })
        .enumerate()
        .map(|(i, offset)| (i + 1, offset))
        .collect();

    // Convert the filtered lints into our `FormattedLintOutput` format.
    let mut lint_outputs: Vec<FormattedLintOutput> = final_lints
        .iter()
        .filter_map(|lint| {
            // Use the start and end fields of the span to create a valid range for indexing.
            let lint_range = lint.span.start..lint.span.end;
            let linted_string = request.text.get(lint_range.clone())?.to_string();

            let containing_para = para_offsets
                .iter()
                .rfind(|(_, para_start_offset)| lint.span.start >= *para_start_offset)?;

            let (para_num, para_start_offset) = containing_para;

            // Make the lint's coordinates relative to its paragraph.
            let relative_start = lint.span.start - para_start_offset;
            let relative_end = lint.span.end - para_start_offset;

            Some(FormattedLintOutput {
                start: relative_start,
                length: relative_end - relative_start,
                end: relative_end,
                paragraph_key: para_num.to_string(),
                paragraph: request.text
                    .lines()
                    .nth(para_num - 1)
                    .unwrap_or("")
                    .to_string(),
                string: linted_string,
                r#type: lint.lint_kind.to_string(),
                suggestions: Suggestions {
                    recommendation: lint.suggestions.iter().map(|s| s.to_string()).collect(),
                },
            })
        })
        .collect();

    lint_outputs.sort_by(|a, b| {
        let para_a = a.paragraph_key.parse::<usize>().unwrap_or(0);
        let para_b = b.paragraph_key.parse::<usize>().unwrap_or(0);
        
        match para_a.cmp(&para_b) {
            Ordering::Equal => a.start.cmp(&b.start),
            other => other,
        }
    });

    HttpResponse::Ok().json(lint_outputs)
}



// {
//   "text": "There are some cases where the the standard grammar\ncheckers don't cut it. That s where Harper comes in handy.\n\nHarper is an language checker for developers. it can detect\nimproper capitalization and misspellled words,\nas well as a number of other issues.\nLike if you break up words you shoul dn't.\nHarper can be a lifesaver when writing technical documents, \nemails or other formal forms of communication.\n\nHarper works everywhere, even offline. Since your data\nnever leaves your device, you don't need to worry aout us\nselling it or using it to train large language models, \ndespite of the consequences.\n\nThe best part: Harper can give you feedback instantly.\nFor most documents, Harper can serve up suggestions in\nunder 10 ms.\n"
// }

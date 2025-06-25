use actix_web::{web, HttpResponse, Responder};
use harper_core::{Document, FstDictionary, Dialect, linting::LintGroup, linting::Linter};
use serde::{Deserialize, Serialize};


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


/// This is the main handler for the `/lint` endpoint.
/// It takes a JSON object with a `text` field, runs the linter,
/// and returns a JSON array of `FormattedLintOutput` objects with paragraph-relative indices.
pub async fn lint_text(request: web::Json<LintRequest>) -> impl Responder {
    // Create a new document from the input text.
    let document = Document::new_plain_english_curated(&request.text);
   
    // Create a linter.
    // TODO: Consider making Dialect configurable or detecting it.
    // TODO: FstDictionary::curated() might be expensive to call repeatedly.
    // Consider creating it once and sharing it (e.g., using web::Data).
    let dictionary = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dictionary, Dialect::American);


    // Get lints from the linter.
    let lints_from_linter = linter.lint(&document);

    // Generate paragraph boundaries. A "paragraph" is defined as a sequence of characters separated by a newline.
    // We store the paragraph number (1-indexed) and its starting byte offset.
    let para_offsets: Vec<(usize, usize)> = request
        .text
        .split('\n')
        .scan(0, |offset, para_text| {
            let current_offset = *offset;
            *offset += para_text.len() + 1; // +1 for the newline character
            Some(current_offset)
        })
        .enumerate()
        .map(|(i, offset)| (i + 1, offset))
        .collect();

    // Convert the lints into our `FormattedLintOutput` format.
    let lint_outputs: Vec<FormattedLintOutput> = lints_from_linter
        .iter()
        .filter_map(|lint| {
            // Extract the original text slice that the lint applies to.
            // We use the `start` and `end` fields of the span to create a valid range.
            let linted_string = request.text.get(lint.span.start..lint.span.end)?.to_string();

            // Find the paragraph that contains this lint.
            // We search in reverse because lints appear within the most recent paragraph.
            let containing_para = para_offsets
                .iter()
                .rfind(|(_, para_start_offset)| lint.span.start >= *para_start_offset)?;

            let (para_num, para_start_offset) = containing_para;

            // Make the lint's coordinates relative to the start of its paragraph.
            let relative_start = lint.span.start - para_start_offset;
            let relative_end = lint.span.end - para_start_offset;

            Some(FormattedLintOutput {
                start: relative_start,
                length: relative_end - relative_start,
                end: relative_end,
                paragraph_key: para_num.to_string(), // Use the dynamically found key.
                string: linted_string,
                r#type: lint.lint_kind.to_string(),
                suggestions: Suggestions {
                    recommendation: lint.suggestions.iter().map(|s| s.to_string()).collect(),
                },
            })
        })
        .collect();


    // Return the lint outputs as a JSON response.
    HttpResponse::Ok().json(lint_outputs)
}

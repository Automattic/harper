use actix_web::{web, HttpResponse, Responder};
use harper_core::{Document, FstDictionary, Dialect, linting::LintGroup, linting::Linter};
use serde::{Deserialize, Serialize};

/// This struct represents a single lint suggestion in the JSON output.
#[derive(Serialize)]
pub struct LintOutput {
    /// The starting byte index of the linted text.
    pub start: usize,
    /// The length of the linted text in bytes.
    pub length: usize,
    /// A human-readable message describing the issue.
    pub issue: String,
    /// The category or type of the lint.
    pub r#type: String, // `type` is a keyword in Rust, so we use `r#type`
    /// A list of suggested replacements for the linted text.
    pub suggestions: Vec<String>,
}

/// This struct represents the incoming JSON request body.
#[derive(Deserialize)]
pub struct LintRequest {
    pub text: String,
}

/// This is the main handler for the `/lint` endpoint.
/// It takes a JSON object with a `text` field, runs the linter,
/// and returns a JSON array of `LintOutput` objects.
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

    // Convert the lints into our `LintOutput` format.
    let lint_outputs: Vec<LintOutput> = lints_from_linter
        .iter()
        .map(|lint| LintOutput {
            start: lint.span.start,
            length: lint.span.end - lint.span.start,
            issue: lint.lint_kind.to_string(), // Changed from message()
            r#type: lint.lint_kind.to_string(),
            suggestions: lint.suggestions.iter().map(|s| s.to_string()).collect(), // Changed from s.text.clone()
        })
        .collect();

    // Return the lint outputs as a JSON response.
    HttpResponse::Ok().json(lint_outputs)
}
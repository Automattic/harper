use actix_web::{web, HttpResponse, Responder};
use harper_core::Document;
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
    let mut document = Document::new_from_str(&request.text);
    // Run all the available linters on the document.
    document.run_all_linters();

    // Convert the lints into our `LintOutput` format.
    let lint_outputs: Vec<LintOutput> = document
        .lints()
        .iter()
        .map(|lint| LintOutput {
            start: lint.span.start,
            length: lint.span.end - lint.span.start,
            issue: lint.lint_kind.message(),
            r#type: lint.lint_kind.to_string(),
            suggestions: lint.suggestions.iter().map(|s| s.text.clone()).collect(),
        })
        .collect();

    // Return the lint outputs as a JSON response.
    HttpResponse::Ok().json(lint_outputs)
}
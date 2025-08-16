use std::collections::HashMap;

use harper_core::linting::{Lint, Suggestion};
use harper_core::{CharStringExt, Document};
use harper_stats::RecordKind;
use serde_json::Value;
use tower_lsp_server::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, TextEdit, Uri,
    WorkspaceEdit,
};

use crate::config::{CodeActionConfig, DiagnosticSeverity};
use crate::pos_conv::span_to_range;

pub fn lints_to_diagnostics(
    source: &[char],
    lints: &[Lint],
    severity: DiagnosticSeverity,
) -> Vec<Diagnostic> {
    lints
        .iter()
        .map(|lint| lint_to_diagnostic(lint, source, severity))
        .collect()
}

pub fn lint_to_code_actions<'a>(
    lint: &'a Lint,
    uri: &'a Uri,
    document: &Document,
    config: &CodeActionConfig,
) -> Vec<CodeActionOrCommand> {
    let mut results = Vec::new();
    let source = document.get_source();

    results.extend(
        lint.suggestions
            .iter()
            .flat_map(|suggestion| {
                let range = span_to_range(source, lint.span);

                let replace_string = match suggestion {
                    Suggestion::ReplaceWith(with) => with.iter().collect(),
                    Suggestion::Remove => "".to_string(),
                    Suggestion::InsertAfter(with) => format!(
                        "{}{}",
                        lint.span.get_content_string(source),
                        with.to_string()
                    ),
                };

                Some(CodeAction {
                    title: suggestion.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range,
                                new_text: replace_string,
                            }],
                        )])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: Some(Command {
                        title: "Record lint statistic".to_owned(),
                        command: "HarperRecordLint".to_owned(),
                        arguments: Some(vec![Value::String(
                            serde_json::to_string(&RecordKind::from_lint(lint, document)).unwrap(),
                        )]),
                    }),
                    is_preferred: None,
                    disabled: None,
                    data: None,
                })
            })
            .map(CodeActionOrCommand::CodeAction),
    );

    results.push(CodeActionOrCommand::Command(Command {
        title: "Ignore Harper error.".to_owned(),
        command: "HarperIgnoreLint".to_owned(),
        arguments: Some(vec![
            serde_json::Value::String(uri.to_string()),
            serde_json::to_value(lint).unwrap(),
        ]),
    }));

    if lint.lint_kind.is_spelling() {
        let orig = lint.span.get_content_string(source);

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{orig}\" to the global dictionary."),
            "HarperAddToUserDict".to_string(),
            Some(vec![orig.clone().into(), uri.to_string().into()]),
        )));

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{orig}\" to the workspace dictionary."),
            "HarperAddToWSDict".to_string(),
            Some(vec![orig.clone().into(), uri.to_string().into()]),
        )));

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{orig}\" to the file dictionary."),
            "HarperAddToFileDict".to_string(),
            Some(vec![orig.into(), uri.to_string().into()]),
        )));
    }

    if config.force_stable {
        results.reverse();
    }

    results
}

fn lint_to_diagnostic(lint: &Lint, source: &[char], severity: DiagnosticSeverity) -> Diagnostic {
    let range = span_to_range(source, lint.span);

    Diagnostic {
        range,
        severity: Some(severity.to_lsp()),
        code: None,
        code_description: None,
        source: Some("Harper".to_string()),
        message: lint.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

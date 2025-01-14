use std::collections::HashMap;

use harper_core::linting::{Lint, Suggestion};
use harper_core::CharStringExt;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, TextEdit, Url,
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
    url: &'a Url,
    source: &'a [char],
    config: &CodeActionConfig,
) -> Vec<CodeActionOrCommand> {
    let mut results = Vec::new();

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
                            url.clone(),
                            vec![TextEdit {
                                range,
                                new_text: replace_string,
                            }],
                        )])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: None,
                    disabled: None,
                    data: None,
                })
            })
            .map(CodeActionOrCommand::CodeAction),
    );

    if lint.lint_kind.is_spelling() {
        let orig = lint.span.get_content_string(source);

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{orig}\" to the global dictionary."),
            "HarperAddToUserDict".to_string(),
            Some(vec![orig.clone().into(), url.to_string().into()]),
        )));

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{orig}\" to the file dictionary."),
            "HarperAddToFileDict".to_string(),
            Some(vec![orig.into(), url.to_string().into()]),
        )));

        if config.force_stable {
            results.reverse();
        }
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

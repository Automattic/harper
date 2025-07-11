use crate::config::{CodeActionConfig, DiagnosticSeverity};
use crate::diagnostics::{lint_to_code_actions, lints_to_diagnostics};
use crate::pos_conv::range_to_span;
use harper_core::linting::{Lint, LintGroup, Linter};
use harper_core::spell::{MergedDictionary, MutableDictionary};
use harper_core::{Document, IgnoredLints, TokenKind, remove_overlaps};
use harper_core::{Lrc, Token};
use tower_lsp_server::lsp_types::{CodeActionOrCommand, Command, Diagnostic, Range, Uri};

pub struct DocumentState {
    pub document: Document,
    pub ident_dict: Lrc<MutableDictionary>,
    pub dict: Lrc<MergedDictionary>,
    pub linter: LintGroup,
    pub language_id: Option<String>,
    pub ignored_lints: IgnoredLints,
    pub uri: Uri,
}

impl DocumentState {
    pub fn ignore_lint(&mut self, lint: &Lint) {
        self.ignored_lints.ignore_lint(lint, &self.document);
    }

    pub fn generate_diagnostics(&mut self, severity: DiagnosticSeverity) -> Vec<Diagnostic> {
        let temp = self.linter.config.clone();
        self.linter.config.fill_with_curated();

        let mut lints = self.linter.lint(&self.document);

        self.linter.config = temp;

        remove_overlaps(&mut lints);
        self.ignored_lints
            .remove_ignored(&mut lints, &self.document);

        lints_to_diagnostics(self.document.get_full_content(), &lints, severity)
    }

    /// Generate code actions results for a selected area.
    pub fn generate_code_actions(
        &mut self,
        range: Range,
        code_action_config: &CodeActionConfig,
    ) -> Vec<CodeActionOrCommand> {
        let temp = self.linter.config.clone();
        self.linter.config.fill_with_curated();

        let mut lints = self.linter.lint(&self.document);

        self.linter.config = temp;

        self.ignored_lints
            .remove_ignored(&mut lints, &self.document);

        lints.sort_by_key(|l| l.priority);

        let source_chars = self.document.get_full_content();

        // Find lints whole span overlaps with range
        let span = range_to_span(source_chars, range).with_len(1);

        let mut actions: Vec<CodeActionOrCommand> = lints
            .into_iter()
            .filter(|lint| lint.span.overlaps_with(span))
            .flat_map(|lint| {
                lint_to_code_actions(&lint, &self.uri, &self.document, code_action_config)
            })
            .collect();

        if let Some(Token {
            kind: TokenKind::Url,
            span,
            ..
        }) = self.document.get_token_at_char_index(span.start)
        {
            actions.push(CodeActionOrCommand::Command(Command::new(
                "Open URL".to_string(),
                "HarperOpen".to_string(),
                Some(vec![self.document.get_span_content_str(span).into()]),
            )))
        }

        actions
    }
}

impl Default for DocumentState {
    fn default() -> Self {
        Self {
            document: Default::default(),
            ident_dict: Default::default(),
            dict: Default::default(),
            linter: Default::default(),
            language_id: Default::default(),
            ignored_lints: Default::default(),
            uri: "https://example.net".parse().unwrap(),
        }
    }
}

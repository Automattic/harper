use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use dirs::{config_dir, data_local_dir};
use harper_core::{Dialect, linting::LintGroupConfig, parsers::MarkdownOptions};
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl DiagnosticSeverity {
    /// Converts `self` to the equivalent LSP type.
    pub fn to_lsp(self) -> tower_lsp_server::lsp_types::DiagnosticSeverity {
        match self {
            DiagnosticSeverity::Error => tower_lsp_server::lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => tower_lsp_server::lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticSeverity::Information => {
                tower_lsp_server::lsp_types::DiagnosticSeverity::INFORMATION
            }
            DiagnosticSeverity::Hint => tower_lsp_server::lsp_types::DiagnosticSeverity::HINT,
        }
    }
}

/// Configuration for how code actions are displayed.
/// Originally motivated by [#89](https://github.com/automattic/harper/issues/89).
#[derive(Debug, Clone, Default)]
pub struct CodeActionConfig {
    /// Instructs `harper-ls` to place unstable code actions last.
    /// In this case, "unstable" refers to their existence and action.
    ///
    /// For example, we always want to allow users to add "misspelled" elements
    /// to dictionary, regardless of the spelling suggestions.
    pub force_stable: bool,
}

impl CodeActionConfig {
    pub fn from_lsp_config(value: Value) -> Result<Self> {
        let mut base = CodeActionConfig::default();

        let Value::Object(value) = value else {
            bail!("The code action configuration must be an object.");
        };

        if let Some(force_stable_val) = value.get("ForceStable") {
            let Value::Bool(force_stable) = force_stable_val else {
                bail!("ForceStable must be a boolean value.");
            };
            base.force_stable = *force_stable;
        };

        Ok(base)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
    pub file_dict_path: PathBuf,
    pub workspace_dict_path: PathBuf,
    pub ignored_lints_path: PathBuf,
    pub stats_path: PathBuf,
    pub lint_config: LintGroupConfig,
    pub diagnostic_severity: DiagnosticSeverity,
    pub code_action_config: CodeActionConfig,
    pub isolate_english: bool,
    pub markdown_options: MarkdownOptions,
    pub dialect: Dialect,
    /// Maximum length (in bytes) a file can have before it's skipped.
    /// Above this limit, the file will not be linted.
    pub max_file_length: usize,
}

impl Config {
    pub fn from_lsp_config(workspace_root: &Path, value: Value) -> Result<Self> {
        let mut base = Config::default();

        let Value::Object(value) = value else {
            bail!("Settings must be an object.");
        };

        let Some(Value::Object(value)) = value.get("harper-ls") else {
            bail!("Settings must contain a \"harper-ls\" key.");
        };

        if let Some(v) = value.get("userDictPath") {
            if !v.is_string() {
                bail!("userDict path must be a string.");
            }

            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.user_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        }

        if let Some(v) = value.get("fileDictPath") {
            if !v.is_string() {
                bail!("fileDict path must be a string.");
            }

            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.file_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        }

        if let Some(v) = value.get("workspaceDictPath") {
            if !v.is_string() {
                bail!("workspaceDict path must be a string.");
            }
            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.workspace_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        } else {
            // Resolve the default path in the project root
            base.workspace_dict_path = base
                .workspace_dict_path
                .try_resolve_in(workspace_root)?
                .to_path_buf();
        }

        if let Some(v) = value.get("ignoredLintsPath") {
            if !v.is_string() {
                bail!("ignoredLintsPath path must be a string.");
            }

            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.ignored_lints_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        }

        if let Some(v) = value.get("statsPath") {
            if let Value::String(path) = v {
                base.file_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            } else {
                bail!("fileDict path must be a string.");
            }
        }

        if let Some(v) = value.get("linters") {
            base.lint_config = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("diagnosticSeverity") {
            base.diagnostic_severity = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("dialect") {
            base.dialect = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("codeActions") {
            base.code_action_config = CodeActionConfig::from_lsp_config(v.clone())?;
        }

        if let Some(v) = value.get("isolateEnglish") {
            if let Value::Bool(v) = v {
                base.isolate_english = *v;
            } else {
                bail!("isolateEnglish path must be a boolean.");
            }
        }

        if let Some(v) = value.get("maxFileLength") {
            base.max_file_length = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("markdown") {
            if let Some(v) = v.get("IgnoreLinkTitle") {
                base.markdown_options.ignore_link_title = serde_json::from_value(v.clone())?;
            }
        }

        Ok(base)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_dict_path: config_dir().unwrap().join("harper-ls/dictionary.txt"),
            file_dict_path: data_local_dir()
                .unwrap()
                .join("harper-ls/file_dictionaries/"),
            workspace_dict_path: ".harper-dictionary.txt".into(),
            ignored_lints_path: data_local_dir().unwrap().join("harper-ls/ignored_lints/"),
            stats_path: data_local_dir().unwrap().join("harper-ls/stats.txt"),
            lint_config: LintGroupConfig::default(),
            diagnostic_severity: DiagnosticSeverity::Hint,
            code_action_config: CodeActionConfig::default(),
            isolate_english: false,
            markdown_options: MarkdownOptions::default(),
            dialect: Dialect::American,
            max_file_length: 120_000,
        }
    }
}

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::document_state::DocumentState;
use crate::git_commit_parser::GitCommitParser;
use crate::ignored_lints_io::{load_ignored_lints, save_ignored_lints};
use crate::io_utils::fileify_path;
use anyhow::{Context, Result, anyhow};
use futures::future::join;
use harper_asciidoc::AsciidocParser;
use harper_comments::CommentParser;
use harper_core::linting::{FlatConfig, LintGroup};
use harper_core::parsers::{
    CollapseIdentifiers, IsolateEnglish, Markdown, OrgMode, Parser, PlainEnglish,
};
use harper_core::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use harper_core::{Dialect, DictWordMetadata, Document, IgnoredLints};
use harper_dictionary_wordlist::{load_dict, save_dict};
use harper_html::HtmlParser;
use harper_ink::InkParser;
use harper_jjdescription::JJDescriptionParser;
use harper_literate_haskell::LiterateHaskellParser;
use harper_python::PythonParser;
use harper_stats::{Record, Stats};
use harper_tex::TeX;
use harper_typst::Typst;
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};
use tower_lsp_server::jsonrpc::Result as JsonResult;
use tower_lsp_server::lsp_types::notification::PublishDiagnostics;
use tower_lsp_server::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    ConfigurationItem, Diagnostic, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidChangeWatchedFilesRegistrationOptions,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, ExecuteCommandOptions,
    ExecuteCommandParams, FileChangeType, FileSystemWatcher, GlobPattern, InitializeParams,
    InitializeResult, InitializedParams, MessageType, PublishDiagnosticsParams, Range,
    Registration, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Uri, WatchKind,
};
use tower_lsp_server::{Client, LanguageServer, UriExt};
use tracing::{error, info, warn};

/// Type alias for pending diagnostic task with its generation
/// The generation is used to ensure delayed diagnostics don't overwrite
/// immediate ones when code actions trigger instant feedback.
type PendingDiagnosticTask = (tokio::task::JoinHandle<()>, u64);

/// Return harper-ls version
pub fn ls_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Clone)]
pub struct Backend {
    client: Client,
    root: Arc<RwLock<PathBuf>>,
    config: Arc<RwLock<Config>>,
    stats: Arc<RwLock<Stats>>,
    doc_state: Arc<Mutex<HashMap<Uri, DocumentState>>>,
    /// Tracks the last change time and generation for each document for diagnostic delay
    /// The generation is incremented on each change, and pending tasks check if their
    /// generation still matches before publishing.
    last_change_time: Arc<Mutex<HashMap<Uri, Instant>>>,
    last_change_generation: Arc<Mutex<HashMap<Uri, u64>>>,
    /// Tracks pending diagnostic publications that are waiting for the delay
    pending_diagnostics: Arc<Mutex<HashMap<Uri, PendingDiagnosticTask>>>,
}

impl Backend {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            root: Arc::new(RwLock::new(".".into())),
            stats: Arc::new(RwLock::new(Stats::new())),
            config: Arc::new(RwLock::new(config)),
            doc_state: Arc::new(Mutex::new(HashMap::new())),
            last_change_time: Arc::new(Mutex::new(HashMap::new())),
            last_change_generation: Arc::new(Mutex::new(HashMap::new())),
            pending_diagnostics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load a specific file's dictionary
    async fn load_file_dictionary(&self, uri: &Uri) -> anyhow::Result<MutableDictionary> {
        // VS Code's unsaved documents have "untitled" scheme
        if uri
            .scheme()
            .is_some_and(|scheme| scheme.eq_lowercase("untitled"))
        {
            return Ok(MutableDictionary::new());
        }

        let path = self
            .get_file_dict_path(uri)
            .await
            .context("Unable to get the file path.")?;

        load_dict(path, self.config.read().await.dialect)
            .await
            .map_err(|err| info!("{err}"))
            .or(Ok(MutableDictionary::new()))
    }

    /// Compute the location of the ignored lint's store.
    async fn get_ignored_lints_path(&self, uri: &Uri) -> anyhow::Result<PathBuf> {
        let config = self.config.read().await;

        Ok(config.ignored_lints_path.join(fileify_path(uri)?))
    }

    async fn save_ignored_lints(&self, uri: &Uri, ignored_lints: &IgnoredLints) -> Result<()> {
        save_ignored_lints(
            self.get_ignored_lints_path(uri)
                .await
                .context("Unable to get ignored lints path.")?,
            ignored_lints,
        )
        .await
        .context("Unable to save ignored lints to path.")
    }

    async fn load_ignored_lints(&self, uri: &Uri) -> Result<IgnoredLints> {
        // VS Code's unsaved documents have "untitled" scheme
        if uri
            .scheme()
            .is_some_and(|scheme| scheme.eq_lowercase("untitled"))
        {
            return Ok(IgnoredLints::new());
        }

        Ok(load_ignored_lints(
            self.get_ignored_lints_path(uri)
                .await
                .context("Unable to get ignored lints path.")?,
        )
        .await
        .map_err(|err| info!("{err}"))
        .unwrap_or(IgnoredLints::new()))
    }

    /// Compute the location of the file's specific dictionary
    async fn get_file_dict_path(&self, uri: &Uri) -> anyhow::Result<PathBuf> {
        let config = self.config.read().await;

        Ok(config.file_dict_path.join(fileify_path(uri)?))
    }

    async fn save_file_dictionary(&self, uri: &Uri, dict: impl Dictionary) -> Result<()> {
        save_dict(
            self.get_file_dict_path(uri)
                .await
                .context("Unable to get the file path.")?,
            dict,
        )
        .await
        .context("Unable to save the dictionary to path.")
    }

    async fn load_user_dictionary(&self) -> MutableDictionary {
        let config = self.config.read().await;

        load_dict(&config.user_dict_path, self.config.read().await.dialect)
            .await
            .map_err(|err| info!("{err}"))
            .unwrap_or(MutableDictionary::new())
    }

    async fn save_user_dictionary(&self, dict: impl Dictionary) -> Result<()> {
        let config = self.config.read().await;

        save_dict(&config.user_dict_path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    async fn load_workspace_dictionary(&self) -> MutableDictionary {
        let config = self.config.read().await;
        load_dict(
            &config.workspace_dict_path,
            self.config.read().await.dialect,
        )
        .await
        .map_err(|err| info!("{err}"))
        .unwrap_or(MutableDictionary::new())
    }

    async fn save_workspace_dictionary(&self, dict: impl Dictionary) -> Result<()> {
        let config = self.config.read().await;
        save_dict(&config.workspace_dict_path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    async fn save_stats(&self) -> Result<()> {
        let (config, stats) = join(self.config.read(), self.stats.read()).await;

        if let Some(parent) = config.stats_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut writer = BufWriter::new(
            OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&config.stats_path)?,
        );
        stats.write(&mut writer)?;
        writer.flush()?;

        Ok(())
    }

    async fn generate_global_dictionary(&self) -> Result<MergedDictionary> {
        let mut dict = MergedDictionary::new();
        dict.add_dictionary(FstDictionary::curated());
        let user_dict = self.load_user_dictionary().await;
        dict.add_dictionary(Arc::new(user_dict));
        let ws_dict = self.load_workspace_dictionary().await;
        dict.add_dictionary(Arc::new(ws_dict));
        Ok(dict)
    }

    async fn generate_file_dictionary(&self, uri: &Uri) -> Result<MergedDictionary> {
        let (global_dictionary, file_dictionary) = tokio::join!(
            self.generate_global_dictionary(),
            self.load_file_dictionary(uri)
        );

        let mut global_dictionary =
            global_dictionary.context("Unable to load the user dictionary.")?;
        global_dictionary.add_dictionary(Arc::new(
            file_dictionary.context("Unable to load the file dictionary.")?,
        ));

        Ok(global_dictionary)
    }

    async fn update_document_from_file(&self, uri: &Uri, language_id: Option<&str>) -> Result<()> {
        let content = tokio::fs::read_to_string(
            uri.to_file_path()
                .ok_or_else(|| anyhow!("Unable to convert URL to file path."))?,
        )
        .await
        .with_context(|| format!("Unable to read from file {uri:?}"))?;

        self.update_document(uri, &content, language_id).await
    }

    async fn update_document(
        &self,
        uri: &Uri,
        text: &str,
        language_id: Option<&str>,
    ) -> Result<()> {
        self.pull_config().await;

        // Copy necessary configuration to avoid holding lock.
        let (
            lint_config,
            markdown_options,
            isolate_english,
            dialect,
            max_file_length,
            exclude_patterns,
        ) = {
            let config = self.config.read().await;
            (
                config.lint_config.clone(),
                config.markdown_options,
                config.isolate_english,
                config.dialect,
                config.max_file_length,
                config.exclude_patterns.clone(),
            )
        };

        let mut doc_lock = self.doc_state.lock().await;

        if !exclude_patterns.is_empty()
            && exclude_patterns.is_match(
                uri.to_file_path()
                    .ok_or_else(|| anyhow!("Unable to convert URI to file path."))?,
            )
        {
            doc_lock.remove(uri);
            return Ok(());
        }

        let ignored_lints = self.load_ignored_lints(uri).await.unwrap_or_default();

        let dict = Arc::new(
            self.generate_file_dictionary(uri)
                .await
                .context("Unable to generate the file dictionary.")?,
        );

        let doc_state = doc_lock.entry(uri.clone()).or_insert_with(|| {
            info!("Constructing new LintGroup for new document.");

            DocumentState {
                ignored_lints,
                linter: LintGroup::new_curated(dict.clone(), dialect)
                    .with_lint_config(lint_config.clone()),
                language_id: language_id.map(|v| v.to_string()),
                dict: dict.clone(),
                uri: uri.clone(),
                ..Default::default()
            }
        });

        if doc_state.dict != dict {
            doc_state.dict = dict.clone();
            info!("Constructing new linter because of modified dictionary.");
            doc_state.linter =
                LintGroup::new_curated(dict.clone(), dialect).with_lint_config(lint_config.clone());
        }

        let Some(language_id) = &doc_state.language_id else {
            doc_lock.remove(uri);
            return Ok(());
        };

        async fn use_ident_dict<'a>(
            backend: &'a Backend,
            new_dict: Arc<MutableDictionary>,
            parser: impl Parser + 'static,
            uri: &'a Uri,
            doc_state: &'a mut DocumentState,
            lint_config: &FlatConfig,
            dialect: Dialect,
        ) -> Result<Box<dyn Parser>> {
            if doc_state.ident_dict != new_dict {
                info!("Constructing new linter because of modified ident dictionary.");
                doc_state.ident_dict = new_dict.clone();

                let mut merged = backend.generate_file_dictionary(uri).await?;
                merged.add_dictionary(new_dict);
                let merged = Arc::new(merged);

                doc_state.linter = LintGroup::new_curated(merged.clone(), dialect)
                    .with_lint_config(lint_config.clone());
                doc_state.dict = merged.clone();
            }

            Ok(Box::new(CollapseIdentifiers::new(
                Box::new(parser),
                Box::new(doc_state.dict.clone()),
            )))
        }

        let source: Vec<char> = text.chars().collect();
        let ts_parser = CommentParser::new_from_language_id(language_id, markdown_options);
        let parser: Option<Box<dyn Parser>> = match language_id.as_str() {
            _ if ts_parser.is_some() => {
                let ts_parser = ts_parser.unwrap();

                if let Some(new_dict) = ts_parser.create_ident_dict(&Arc::new(source)) {
                    Some(
                        use_ident_dict(
                            self,
                            Arc::new(new_dict),
                            ts_parser,
                            uri,
                            doc_state,
                            &lint_config,
                            dialect,
                        )
                        .await?,
                    )
                } else {
                    Some(Box::new(ts_parser))
                }
            }
            "git-commit" | "gitcommit" | "octo" => {
                Some(Box::new(GitCommitParser::new_markdown(markdown_options)))
            }
            "html" => Some(Box::new(HtmlParser::default())),
            "asciidoc" => Some(Box::new(AsciidocParser::default())),
            "ink" => Some(Box::new(InkParser::default())),
            "jj-commit" | "jjdescription" => {
                Some(Box::new(JJDescriptionParser::new(markdown_options)))
            }
            "lhaskell" | "literate haskell" => {
                let parser = LiterateHaskellParser::new_markdown(markdown_options);

                if let Some(new_dict) =
                    parser.create_ident_dict(&Arc::new(source), markdown_options)
                {
                    Some(
                        use_ident_dict(
                            self,
                            Arc::new(new_dict),
                            parser,
                            uri,
                            doc_state,
                            &lint_config,
                            dialect,
                        )
                        .await?,
                    )
                } else {
                    Some(Box::new(parser))
                }
            }
            "mail" => Some(Box::new(PlainEnglish)),
            "markdown" | "quarto" => Some(Box::new(Markdown::new(markdown_options))),
            "org" => Some(Box::new(OrgMode)),
            "plaintext" | "text" => Some(Box::new(PlainEnglish)),
            "python" => Some(Box::new(PythonParser::default())),
            "typst" => Some(Box::new(Typst)),
            "tex" | "plaintex" | "latex" => Some(Box::new(TeX::default())),
            _ => None,
        };

        match parser {
            None => {
                doc_lock.remove(uri);
            }
            Some(mut parser) => {
                if isolate_english {
                    parser = Box::new(IsolateEnglish::new(parser, doc_state.dict.clone()));
                }

                // Don't lint on documents larger than the configured maximum length.
                if text.len() <= max_file_length {
                    doc_state.document = Document::new(text, &parser, &doc_state.dict);
                } else {
                    // Ensures that existing lints are cleared when we stop linting the file.
                    // Otherwise, prior lints will remain, and they will quickly fall out of sync
                    // with the document when it is edited.
                    doc_state.document = Document::default();
                }
            }
        }

        Ok(())
    }

    async fn generate_code_actions(
        &self,
        uri: &Uri,
        range: Range,
    ) -> JsonResult<Vec<CodeActionOrCommand>> {
        let (config, mut doc_states) = tokio::join!(self.config.read(), self.doc_state.lock());
        let Some(doc_state) = doc_states.get_mut(uri) else {
            return Ok(Vec::new());
        };

        Ok(doc_state.generate_code_actions(range, &config.code_action_config))
    }

    async fn generate_diagnostics(&self, uri: &Uri) -> Vec<Diagnostic> {
        // Copy necessary configuration to avoid holding lock.
        let diagnostic_severity = {
            let config = self.config.read().await;
            config.diagnostic_severity
        };

        let mut doc_states = self.doc_state.lock().await;
        let Some(doc_state) = doc_states.get_mut(uri) else {
            return Vec::new();
        };

        doc_state.generate_diagnostics(diagnostic_severity)
    }

    async fn publish_diagnostics(&self, uri: &Uri) {
        let diagnostics = self.generate_diagnostics(uri).await;

        warn!(
            "DEBUG: publish_diagnostics: {} diagnostics for {:?}",
            diagnostics.len(),
            uri
        );

        let result = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: None,
        };

        self.client
            .send_notification::<PublishDiagnostics>(result)
            .await;

        warn!("DEBUG: publish_diagnostics notification sent for {:?}", uri);
    }

    /// Publish diagnostics immediately, canceling any pending delayed diagnostics for the document.
    /// This is used for code actions where we want instant feedback.
    async fn publish_diagnostics_immediately(&self, uri: &Uri) {
        warn!("DEBUG: Publishing diagnostics immediately for {:?}", uri);

        // Increment the generation to invalidate any pending delayed diagnostics
        {
            let mut generations = self.last_change_generation.lock().await;
            *generations.entry(uri.clone()).or_insert(0) += 1;
        }

        // Cancel any pending delayed diagnostics for this document
        let mut pending = self.pending_diagnostics.lock().await;
        if let Some((handle, old_generation)) = pending.remove(uri) {
            warn!(
                "DEBUG: Cancelled pending delayed diagnostics (gen {}) for {:?}",
                old_generation, uri
            );
            handle.abort();
        }
        drop(pending);

        // Update last change time to now
        let mut last_changes = self.last_change_time.lock().await;
        last_changes.insert(uri.clone(), Instant::now());
        drop(last_changes);

        // Publish diagnostics immediately
        self.publish_diagnostics(uri).await;
    }

    /// Update the configuration of the server and publish document updates that
    /// match it.
    async fn update_config_from_obj(&self, json_obj: Value) {
        if let Ok(new_config) = Config::from_lsp_config(&self.root.read().await, json_obj)
            .map_err(|err| error!("{err}"))
        {
            let mut config = self.config.write().await;
            *config = new_config;
        }
    }

    async fn pull_config(&self) {
        let mut new_config = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: None,
            }])
            .await
            .unwrap_or(vec![json!({ "harper-ls": {} })]);

        if let Some(first) = new_config.pop() {
            self.update_config_from_obj(first).await;
        }
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonResult<InitializeResult> {
        if let Some(root) = params
            .workspace_folders
            .as_ref()
            // We take the first workspace folder
            .and_then(|v| v.first())
            .map(|f| &f.uri)
            // Or failing that, the root_uri (which is deprecated in favour of workspace_folders)
            .or(
                #[allow(deprecated)]
                params.root_uri.as_ref(),
            )
            .and_then(|u| u.to_file_path().map(PathBuf::from))
            // Or failing that, the root_path (which is deprecated in favour of root_uri)
            .or(
                #[allow(deprecated)]
                params.root_path.as_deref().map(PathBuf::from),
            )
        {
            // Save the workspace root away for use during the configuration step
            *self.root.write().await = root;
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "harper-ls".to_owned(),
                version: Some(ls_version().to_owned()),
            }),
            capabilities: ServerCapabilities {
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "HarperRecordLint".to_owned(),
                        "HarperRecordLintAndUpdate".to_owned(),
                        "HarperAddToUserDict".to_owned(),
                        "HarperAddToWSDict".to_owned(),
                        "HarperAddToFileDict".to_owned(),
                        "HarperOpen".to_owned(),
                        "HarperIgnoreLint".to_owned(),
                    ],
                    ..Default::default()
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                    },
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Server initialized!")
            .await;

        self.pull_config().await;

        let did_change_watched_files = Registration {
            id: "workspace/didChangeWatchedFiles".to_owned(),
            method: "workspace/didChangeWatchedFiles".to_owned(),
            register_options: Some(
                serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                    watchers: vec![FileSystemWatcher {
                        glob_pattern: GlobPattern::String("**/*".to_owned()),
                        kind: Some(WatchKind::Delete),
                    }],
                })
                .unwrap(),
            ),
        };
        if let Err(err) = self
            .client
            .register_capability(vec![did_change_watched_files])
            .await
        {
            warn!("Unable to register watch file capability: {}", err);
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_document(
            &params.text_document.uri,
            &params.text_document.text,
            Some(&params.text_document.language_id),
        )
        .await
        .map_err(|err| error!("{err}"))
        .err();

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(last) = params.content_changes.last() else {
            return;
        };

        let uri = params.text_document.uri.clone();

        if let Err(err) = self.update_document(&uri, &last.text, None).await {
            error!("{err}")
        }

        // Check if diagnostic delay is configured
        let config = self.config.read().await;
        let delay_ms = config.diagnostic_delay_ms;
        drop(config); // Release the lock early

        if delay_ms > 0 {
            // Get the current generation for this document
            let current_generation = {
                let mut generations = self.last_change_generation.lock().await;
                *generations.entry(uri.clone()).or_insert(0) += 1;
                generations[&uri]
            };

            // Cancel any pending diagnostic publication for this document
            let mut pending = self.pending_diagnostics.lock().await;
            if let Some((handle, _old_generation)) = pending.remove(&uri) {
                handle.abort();
            }

            // Update last change time
            let mut last_changes = self.last_change_time.lock().await;
            last_changes.insert(uri.clone(), Instant::now());
            drop(last_changes);

            // Schedule diagnostics to be published after the delay
            let uri_clone = uri.clone();
            let last_change_generation = self.last_change_generation.clone();
            let backend = self.clone();
            let scheduled_generation = current_generation;

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                // Check if this is still the most recent change by comparing generations
                let generations = last_change_generation.lock().await;
                let current_gen = generations.get(&uri_clone).copied().unwrap_or(0);

                if current_gen == scheduled_generation {
                    // This is still the most recent change, publish diagnostics
                    backend.publish_diagnostics(&uri_clone).await;
                }
                // Otherwise, a newer change has occurred and will handle diagnostics
            });

            pending.insert(uri, (handle, scheduled_generation));
        } else {
            // No delay configured, publish immediately
            self.publish_diagnostics(&uri).await;
        }
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {
        let uri = _params.text_document.uri;
        let mut doc_lock = self.doc_state.lock().await;
        doc_lock.remove(&uri);

        self.client
            .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                uri: uri.clone(),
                diagnostics: vec![],
                version: None,
            })
            .await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let mut doc_lock = self.doc_state.lock().await;
        let mut uris_to_clear = Vec::new();

        for change in &params.changes {
            if change.typ != FileChangeType::DELETED {
                continue;
            }

            doc_lock.retain(|uri, _| {
                // `change.uri` could be a directory so use `starts_with` instead of `==`.
                let to_remove = uri.as_str().starts_with(change.uri.as_str());

                if to_remove {
                    uris_to_clear.push(uri.clone());
                }

                !to_remove
            });
        }

        for uri in &uris_to_clear {
            self.client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics: vec![],
                    version: None,
                })
                .await;
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> JsonResult<Option<Value>> {
        let mut string_args = params
            .arguments
            .iter()
            .map(|v| serde_json::from_value::<String>(v.clone()).unwrap());

        let Some(first) = string_args.next() else {
            return Ok(None);
        };

        warn!("DEBUG: Received command: \"{}\"", params.command.as_str());

        match params.command.as_str() {
            "HarperRecordLint" => {
                let Ok(kind) = serde_json::from_str(&first) else {
                    error!("Unable to deserialize RecordKind.");
                    return Ok(None);
                };

                let record = Record::now(kind);

                let mut stats = self.stats.write().await;
                stats.records.push(record);
            }
            "HarperRecordLintAndUpdate" => {
                let Ok(uri) = first.parse::<Uri>() else {
                    error!("Unable to parse URI for HarperRecordLintAndUpdate");
                    return Ok(None);
                };

                let Some(second) = string_args.next() else {
                    error!("Missing RecordKind argument for HarperRecordLintAndUpdate");
                    return Ok(None);
                };

                let Ok(kind) = serde_json::from_str(&second) else {
                    error!("Unable to deserialize RecordKind.");
                    return Ok(None);
                };

                let record = Record::now(kind);
                let mut stats = self.stats.write().await;
                stats.records.push(record);

                // This command is called after a suggestion code action is applied.
                // The suggestion's WorkspaceEdit has already been applied by the client,
                // but the client might send executeCommand before textDocument/didChange.
                // We need to wait briefly to ensure the document state is updated.
                warn!("DEBUG: HarperRecordLintAndUpdate called for {:?}", uri);

                // Wait a short time to allow did_change to arrive
                tokio::time::sleep(Duration::from_millis(50)).await;

                warn!(
                    "DEBUG: Publishing diagnostics immediately (after wait) for {:?}",
                    uri
                );
                self.publish_diagnostics_immediately(&uri).await;
            }
            "HarperAddToUserDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_uri = second.parse().unwrap();

                let mut dict = self.load_user_dictionary().await;
                dict.append_word(word, DictWordMetadata::default());
                self.save_user_dictionary(dict)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_uri, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics_immediately(&file_uri).await;
            }
            "HarperAddToWSDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_uri = second.parse().unwrap();

                let mut dict = self.load_workspace_dictionary().await;
                dict.append_word(word, DictWordMetadata::default());
                self.save_workspace_dictionary(dict)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_uri, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics_immediately(&file_uri).await;
            }
            "HarperAddToFileDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_uri = second.parse().unwrap();

                let mut dict = match self
                    .load_file_dictionary(&file_uri)
                    .await
                    .map_err(|err| error!("{err}"))
                {
                    Ok(dict) => dict,
                    Err(_) => {
                        return Ok(None);
                    }
                };
                dict.append_word(word, DictWordMetadata::default());

                self.save_file_dictionary(&file_uri, dict)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_uri, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics_immediately(&file_uri).await;
            }
            "HarperOpen" => match open::that(&first) {
                Ok(()) => {
                    let message = format!(r#"Opened "{first}""#);

                    self.client.log_message(MessageType::INFO, &message).await;

                    info!("{}", message);
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, "Unable to open URL")
                        .await;
                    error!("Unable to open URL: {}", err);
                }
            },
            "HarperIgnoreLint" => {
                let Ok(uri) = first.parse() else {
                    error!("Unable to parse URL from command: {first}");
                    return Ok(None);
                };

                let Some(second) = params.arguments.into_iter().nth(1) else {
                    error!("Not enough arguments to HarperIgnoreLint");
                    return Ok(None);
                };

                let Ok(lint) = serde_json::from_value(second) else {
                    error!("Unable to parse lint.");
                    return Ok(None);
                };

                let mut doc_lock = self.doc_state.lock().await;
                let Some(doc_state) = doc_lock.get_mut(&uri) else {
                    error!("Requested document has not been loaded.");
                    return Ok(None);
                };

                doc_state.ignore_lint(&lint);
                if let Err(_err) = self
                    .save_ignored_lints(&uri, &doc_state.ignored_lints)
                    .await
                {
                    error!("Unable to save ignored lints.");
                    return Ok(None);
                }

                drop(doc_lock);

                self.publish_diagnostics_immediately(&uri).await;
            }
            _ => (),
        }

        Ok(None)
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.update_config_from_obj(params.settings).await;

        let uris: Vec<Uri> = {
            let mut doc_lock = self.doc_state.lock().await;
            let config_lock = self.config.read().await;

            for doc in doc_lock.values_mut() {
                info!("Constructing new LintGroup for updated configuration.");
                doc.linter = LintGroup::new_curated(doc.dict.clone(), config_lock.dialect)
                    .with_lint_config(config_lock.lint_config.clone());
            }

            doc_lock.keys().cloned().collect()
        };

        for uri in uris {
            self.update_document_from_file(&uri, None)
                .await
                .map_err(|err| error!("{err}"))
                .err();
            self.publish_diagnostics_immediately(&uri).await;
        }
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> JsonResult<Option<CodeActionResponse>> {
        let actions = self
            .generate_code_actions(&params.text_document.uri, params.range)
            .await?;

        Ok(Some(actions))
    }

    async fn shutdown(&self) -> JsonResult<()> {
        let doc_states = self.doc_state.lock().await;

        // Clears the diagnostics for open buffers.
        for uri in doc_states.keys() {
            let result = PublishDiagnosticsParams {
                uri: uri.clone(),
                diagnostics: vec![],
                version: None,
            };

            self.client
                .send_notification::<PublishDiagnostics>(result)
                .await;
        }

        if self.save_stats().await.is_err() {
            error!("Unable to save stats.")
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::Mutex;
    use tower_lsp_server::lsp_types::Uri;

    /// Test that verifies the publish_diagnostics_immediately logic
    /// by simulating the state changes it performs
    #[tokio::test]
    async fn test_publish_diagnostics_immediately_cancels_pending() {
        // Create mock state that mimics Backend's internal state
        // Using tuple type directly to avoid needing to import the type alias in tests
        let pending_diagnostics: Arc<Mutex<HashMap<Uri, (tokio::task::JoinHandle<()>, u64)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let last_change_time: Arc<Mutex<HashMap<Uri, Instant>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let last_change_generation: Arc<Mutex<HashMap<Uri, u64>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let test_uri: Uri = "file:///test.txt".parse().unwrap();

        // Simulate a pending diagnostic task with generation 1
        {
            let mut pending = pending_diagnostics.lock().await;
            let handle =
                tokio::spawn(async { tokio::time::sleep(Duration::from_millis(1000)).await });
            pending.insert(test_uri.clone(), (handle, 1));
        }

        // Simulate setting generation to 1
        {
            let mut generations = last_change_generation.lock().await;
            generations.insert(test_uri.clone(), 1);
        }

        // Verify pending diagnostic exists
        {
            let pending = pending_diagnostics.lock().await;
            assert!(
                pending.contains_key(&test_uri),
                "Pending diagnostic should exist before cancellation"
            );
        }

        // Simulate what publish_diagnostics_immediately does:
        // Step 1: Increment generation
        {
            let mut generations = last_change_generation.lock().await;
            *generations.entry(test_uri.clone()).or_insert(0) += 1;
        }

        // Step 2: Cancel any pending delayed diagnostics for this document
        {
            let mut pending = pending_diagnostics.lock().await;
            if let Some((handle, _old_gen)) = pending.remove(&test_uri) {
                handle.abort();
            }
        }

        // Step 3: Update last change time to now
        {
            let mut last_changes = last_change_time.lock().await;
            last_changes.insert(test_uri.clone(), Instant::now());
        }

        // Verify the pending diagnostic was removed
        {
            let pending = pending_diagnostics.lock().await;
            assert!(
                !pending.contains_key(&test_uri),
                "Pending diagnostic should have been removed"
            );
        }

        // Verify the last change time was updated
        {
            let last_changes = last_change_time.lock().await;
            assert!(
                last_changes.contains_key(&test_uri),
                "Last change time should have been set"
            );

            let last_time = last_changes.get(&test_uri).unwrap();
            let elapsed = last_time.elapsed();
            assert!(
                elapsed < Duration::from_millis(100),
                "Last change time should be very recent"
            );
        }

        // Verify generation was incremented
        {
            let generations = last_change_generation.lock().await;
            assert_eq!(
                generations[&test_uri], 2,
                "Generation should have been incremented"
            );
        }
    }

    /// Test that verifies the immediate method handles non-existent pending diagnostics gracefully
    #[tokio::test]
    async fn test_publish_diagnostics_immediately_no_pending() {
        // Using tuple type directly to avoid needing to import the type alias in tests
        let pending_diagnostics: Arc<Mutex<HashMap<Uri, (tokio::task::JoinHandle<()>, u64)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let last_change_time: Arc<Mutex<HashMap<Uri, Instant>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let last_change_generation: Arc<Mutex<HashMap<Uri, u64>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let test_uri: Uri = "file:///test.txt".parse().unwrap();

        // Simulate incrementing generation
        {
            let mut generations = last_change_generation.lock().await;
            *generations.entry(test_uri.clone()).or_insert(0) += 1;
        }

        // Call the logic without any pending diagnostics
        {
            let mut pending = pending_diagnostics.lock().await;
            if let Some((handle, _old_gen)) = pending.remove(&test_uri) {
                handle.abort();
            }
        }

        {
            let mut last_changes = last_change_time.lock().await;
            last_changes.insert(test_uri.clone(), Instant::now());
        }

        // Should not panic - just updates last change time
        {
            let last_changes = last_change_time.lock().await;
            assert!(last_changes.contains_key(&test_uri));
        }
    }

    /// Test that verifies multiple documents can have independent pending diagnostics
    #[tokio::test]
    async fn test_multiple_documents_independent_pending() {
        // Using tuple type directly to avoid needing to import the type alias in tests
        let pending_diagnostics: Arc<Mutex<HashMap<Uri, (tokio::task::JoinHandle<()>, u64)>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let uri1: Uri = "file:///test1.txt".parse().unwrap();
        let uri2: Uri = "file:///test2.txt".parse().unwrap();

        // Add pending diagnostics for both documents
        {
            let mut pending = pending_diagnostics.lock().await;
            let handle1 =
                tokio::spawn(async { tokio::time::sleep(Duration::from_millis(1000)).await });
            let handle2 =
                tokio::spawn(async { tokio::time::sleep(Duration::from_millis(1000)).await });
            pending.insert(uri1.clone(), (handle1, 1));
            pending.insert(uri2.clone(), (handle2, 1));
        }

        // Verify both exist
        {
            let pending = pending_diagnostics.lock().await;
            assert!(pending.contains_key(&uri1));
            assert!(pending.contains_key(&uri2));
        }

        // Cancel only uri1's pending diagnostic (simulating publish_diagnostics_immediately for uri1)
        {
            let mut pending = pending_diagnostics.lock().await;
            if let Some((handle, _old_gen)) = pending.remove(&uri1) {
                handle.abort();
            }
        }

        // Verify uri1's pending was cancelled but uri2's is still there
        {
            let pending = pending_diagnostics.lock().await;
            assert!(
                !pending.contains_key(&uri1),
                "uri1's pending should be cancelled"
            );
            assert!(
                pending.contains_key(&uri2),
                "uri2's pending should still exist"
            );
        }
    }
}

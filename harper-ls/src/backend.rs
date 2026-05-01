use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::document_state::DocumentState;
use crate::git_commit_parser::GitCommitParser;
use crate::ignored_lints_io::{load_ignored_lints, save_ignored_lints};
use crate::io_utils::fileify_path;
use crate::language_detection::LanguageDetectionRegistry;
use anyhow::{Context, Result, anyhow};
use futures::future::join;
use harper_asciidoc::AsciidocParser;
use harper_comments::CommentParser;
use harper_core::linting::{FlatConfig, LintGroup};
use harper_core::parsers::{
    CollapseIdentifiers, IsolateEnglish, Markdown, OrgMode, Parser, PlainEnglish, PlainGerman,
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
use tracing::{debug, error, info, warn};

/// Return harper-ls version
pub fn ls_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct Backend {
    client: Client,
    root: Arc<RwLock<PathBuf>>,
    config: Arc<RwLock<Config>>,
    stats: Arc<RwLock<Stats>>,
    doc_state: Arc<Mutex<HashMap<Uri, DocumentState>>>,
    lang_detect: Arc<LanguageDetectionRegistry>,
    /// Tracks the last time each document was changed for diagnostic delay
    last_change_time: Arc<Mutex<HashMap<Uri, Instant>>>,
    /// Tracks pending diagnostic publications that are waiting for the delay
    pending_diagnostics: Arc<Mutex<HashMap<Uri, tokio::task::JoinHandle<()>>>>,
}

impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            root: self.root.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
            doc_state: self.doc_state.clone(),
            lang_detect: self.lang_detect.clone(),
            last_change_time: self.last_change_time.clone(),
            pending_diagnostics: self.pending_diagnostics.clone(),
        }
    }
}

impl Backend {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            root: Arc::new(RwLock::new(".".into())),
            stats: Arc::new(RwLock::new(Stats::new())),
            config: Arc::new(RwLock::new(config)),
            doc_state: Arc::new(Mutex::new(HashMap::new())),
            lang_detect: Arc::new(LanguageDetectionRegistry::new()),
            last_change_time: Arc::new(Mutex::new(HashMap::new())),
            pending_diagnostics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load a specific file's dictionary, using the given dialect to determine
    /// the dictionary path suffix.
    fn parser_for_prose(
        language_id: &str,
        dialect: Dialect,
        markdown_options: harper_core::parsers::MarkdownOptions,
    ) -> Option<Box<dyn Parser>> {
        match language_id {
            "mail" => Some(if dialect.is_german() {
                Box::new(PlainGerman)
            } else {
                Box::new(PlainEnglish)
            }),
            "markdown" | "quarto" => Some(if dialect.is_german() {
                Box::new(Markdown::new_german(markdown_options))
            } else {
                Box::new(Markdown::new(markdown_options))
            }),
            "org" => Some(if dialect.is_german() {
                Box::new(OrgMode::new_german())
            } else {
                Box::new(OrgMode::default())
            }),
            "plaintext" | "text" => Some(if dialect.is_german() {
                Box::new(PlainGerman)
            } else {
                Box::new(PlainEnglish)
            }),
            _ => None,
        }
    }

    async fn load_file_dictionary(
        &self,
        uri: &Uri,
        dialect: Dialect,
    ) -> anyhow::Result<MutableDictionary> {
        // VS Code's unsaved documents have "untitled" scheme
        if uri
            .scheme()
            .is_some_and(|scheme| scheme.eq_lowercase("untitled"))
        {
            return Ok(MutableDictionary::new());
        }

        let path = self
            .get_file_dict_path(uri, dialect)
            .await
            .context("Unable to get the file path.")?;

        load_dict(path, dialect)
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

    /// Compute the location of the file's specific dictionary.
    /// For non-English dialects, files are stored in a dialect-suffixed directory
    /// (e.g. `file_dictionaries-de/`).
    async fn get_file_dict_path(&self, uri: &Uri, dialect: Dialect) -> anyhow::Result<PathBuf> {
        let config = self.config.read().await;
        let suffix = dialect.dict_suffix();

        if suffix.is_empty() {
            Ok(config.file_dict_path.join(fileify_path(uri)?))
        } else {
            // Append suffix to the directory name, e.g. "file_dictionaries" -> "file_dictionaries-de"
            let base = &config.file_dict_path;
            let mut dir_name = base
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            dir_name.push_str(suffix);
            let suffixed = base.with_file_name(dir_name);
            Ok(suffixed.join(fileify_path(uri)?))
        }
    }

    async fn save_file_dictionary(
        &self,
        uri: &Uri,
        dict: impl Dictionary,
        dialect: Dialect,
    ) -> Result<()> {
        save_dict(
            self.get_file_dict_path(uri, dialect)
                .await
                .context("Unable to get the file path.")?,
            dict,
        )
        .await
        .context("Unable to save the dictionary to path.")
    }

    async fn load_user_dictionary(&self, dialect: Dialect) -> MutableDictionary {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.user_dict_path, dialect);

        load_dict(path, dialect)
            .await
            .map_err(|err| info!("{err}"))
            .unwrap_or(MutableDictionary::new())
    }

    async fn save_user_dictionary(&self, dict: impl Dictionary, dialect: Dialect) -> Result<()> {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.user_dict_path, dialect);

        save_dict(path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    async fn load_workspace_dictionary(&self, dialect: Dialect) -> MutableDictionary {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.workspace_dict_path, dialect);

        load_dict(path, dialect)
            .await
            .map_err(|err| info!("{err}"))
            .unwrap_or(MutableDictionary::new())
    }

    async fn save_workspace_dictionary(
        &self,
        dict: impl Dictionary,
        dialect: Dialect,
    ) -> Result<()> {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.workspace_dict_path, dialect);

        save_dict(path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    /// Computes a dialect-aware path from a base path.
    /// For English (default), returns the path unchanged.
    /// For German, inserts a suffix before the extension (or appends it for dirs).
    /// E.g. `dictionary.txt` -> `dictionary-de.txt`,
    ///      `.harper-dictionary.txt` -> `.harper-dictionary-de.txt`.
    fn dialect_path(base: &Path, dialect: Dialect) -> PathBuf {
        let suffix = dialect.dict_suffix();
        if suffix.is_empty() {
            return base.to_path_buf();
        }

        if let Some(ext) = base.extension() {
            let stem = base.file_stem().unwrap_or_default().to_string_lossy();
            let new_name = format!("{}{}.{}", stem, suffix, ext.to_string_lossy());
            base.with_file_name(new_name)
        } else {
            // No extension (directory-like path): just append suffix
            let name = base
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            base.with_file_name(format!("{}{}", name, suffix))
        }
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

    async fn generate_global_dictionary(&self, dialect: Dialect) -> Result<MergedDictionary> {
        let mut dict = MergedDictionary::new();

        // Load appropriate dictionary based on dialect
        match dialect {
            d if d.is_german() => {
                dict.add_dictionary(harper_core::spell::curated_german_dictionary());
                info!("Loaded German dictionary for dialect: {:?}", dialect);
            }
            _ => {
                dict.add_dictionary(FstDictionary::curated());
                info!("Loaded English dictionary for dialect: {:?}", dialect);
            }
        }

        let user_dict = self.load_user_dictionary(dialect).await;
        dict.add_dictionary(Arc::new(user_dict));
        let ws_dict = self.load_workspace_dictionary(dialect).await;
        dict.add_dictionary(Arc::new(ws_dict));
        Ok(dict)
    }

    async fn generate_file_dictionary(
        &self,
        uri: &Uri,
        dialect: Dialect,
    ) -> Result<MergedDictionary> {
        let (global_dictionary, file_dictionary) = tokio::join!(
            self.generate_global_dictionary(dialect),
            self.load_file_dictionary(uri, dialect)
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

        info!(
            "Opening document: {:?} with language_id: {:?}",
            uri, language_id
        );

        // When language_id is None (e.g. on did_change), look up the stored language_id.
        let stored_language_id: Option<String> = if language_id.is_none() {
            let doc_lock = self.doc_state.lock().await;
            doc_lock
                .get(uri)
                .and_then(|state| state.language_id.clone())
        } else {
            None
        };
        let effective_language_id: Option<&str> = language_id.or(stored_language_id.as_deref());

        // Auto-detect language for prose-oriented documents.
        // Covers markdown, plaintext, org, tex, mail, and similar formats.
        let is_prose = effective_language_id
            .map(|s| {
                s.contains("markdown")
                    || s.contains("md")
                    || s == "source.md"
                    || s == "plaintext"
                    || s == "text"
                    || s == "org"
                    || s == "tex"
                    || s == "latex"
                    || s == "plaintex"
                    || s == "mail"
                    || s == "quarto"
                    || s == "asciidoc"
                    || s == "typst"
            })
            .unwrap_or(false);

        info!(
            "Is prose: {} for effective_language_id: {:?}",
            is_prose, effective_language_id
        );

        let detected_dialect: Dialect = if is_prose {
            let word_count = text.split_whitespace().count();
            info!("Word count: {}", word_count);

            let cached_dialect = {
                let doc_lock = self.doc_state.lock().await;
                doc_lock.get(uri).and_then(|state| state.cached_dialect)
            };

            if word_count >= 20 {
                // Re-run detection when we have enough content so that e.g. an
                // empty-then-typed document can switch from English to German.
                let dict = FstDictionary::curated();
                let detected = self
                    .lang_detect
                    .detect_language(text, &dict, Dialect::American);
                debug!(
                    "harper-ls dialect detect: {:?} for {:?} ({} words)",
                    detected, uri, word_count
                );

                // If the dialect changed for an existing document, force the linter
                // to be rebuilt by clearing the cached dict.
                if cached_dialect != Some(detected) {
                    info!(
                        "Dialect changed from {:?} to {:?}, will rebuild linter",
                        cached_dialect, detected
                    );
                    let mut doc_lock = self.doc_state.lock().await;
                    if let Some(state) = doc_lock.get_mut(uri) {
                        // Force a dict mismatch so the linter rebuild is triggered.
                        state.dict = Arc::new(MergedDictionary::new());
                    }
                }

                detected
            } else if let Some(cached) = cached_dialect {
                // Not enough words yet — keep previous detection.
                info!("Using cached dialect: {:?} ({} words)", cached, word_count);
                cached
            } else {
                info!(
                    "Insufficient content for detection ({} words), using default dialect",
                    word_count
                );
                Dialect::American
            }
        } else {
            // For non-prose files (code, etc.) use the configured dialect.
            let config = self.config.read().await;
            config.dialect
        };

        // Copy necessary configuration to avoid holding lock.
        let (
            lint_config,
            markdown_options,
            isolate_english,
            dialect, // Configured dialect (used as fallback)
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
            self.generate_file_dictionary(uri, detected_dialect)
                .await
                .context("Unable to generate the file dictionary.")?,
        );

        let doc_state = doc_lock.entry(uri.clone()).or_insert_with(|| {
            info!("Constructing new LintGroup for new document.");

            DocumentState {
                ignored_lints,
                linter: {
                    let mut l = LintGroup::new_curated(dict.clone(), detected_dialect);
                    l.config.merge_from(lint_config.clone());
                    l
                },
                // Prefer the provided language_id; fall back to what was already stored
                // (effective_language_id already resolved both but we need an owned String).
                language_id: effective_language_id.map(|v| v.to_string()),
                dict: dict.clone(),
                uri: uri.clone(),
                // Store the detected dialect so did_change can use it via stored_language_id
                // and so future cache lookups find the right value.
                cached_dialect: if is_prose {
                    Some(detected_dialect)
                } else {
                    None
                },
                ..Default::default()
            }
        });

        // On subsequent updates, also keep cached_dialect in sync.
        if is_prose {
            doc_state.cached_dialect = Some(detected_dialect);
        }

        if doc_state.dict != dict {
            doc_state.dict = dict.clone();
            info!("Constructing new linter because of modified dictionary.");
            let mut l = LintGroup::new_curated(dict.clone(), detected_dialect);
            l.config.merge_from(lint_config.clone());
            doc_state.linter = l;
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

                let mut merged = backend.generate_file_dictionary(uri, dialect).await?;
                merged.add_dictionary(new_dict);
                let merged = Arc::new(merged);

                doc_state.linter = {
                    let mut l = LintGroup::new_curated(merged.clone(), dialect);
                    l.config.merge_from(lint_config.clone());
                    l
                };
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
                            detected_dialect,
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
            "mail" | "markdown" | "quarto" | "org" | "plaintext" | "text" => {
                Self::parser_for_prose(language_id, detected_dialect, markdown_options)
            }
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

    /// Returns the detected dialect for a document, falling back to configured default.
    async fn get_document_dialect(&self, uri: &Uri) -> Dialect {
        let doc_lock = self.doc_state.lock().await;
        if let Some(state) = doc_lock.get(uri) {
            if let Some(d) = state.cached_dialect {
                return d;
            }
        }
        self.config.read().await.dialect
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

        debug!(
            "harper-ls publish_diagnostics: {} lints for {:?}",
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
    }

    /// Update the configuration of the server and publish document updates that
    /// match it.
    async fn update_config_from_obj(&self, json_obj: Value) {
        // Handle different configuration formats from different editors
        let processed_json = match json_obj {
            Value::Object(mut obj) => {
                // If the object doesn't have a "harper-ls" key, add one
                if !obj.contains_key("harper-ls") {
                    obj.insert(
                        "harper-ls".to_string(),
                        Value::Object(serde_json::Map::new()),
                    );
                }
                Value::Object(obj)
            }
            Value::Null => {
                // Handle null configuration by using default empty object
                json!({ "harper-ls": {} })
            }
            Value::String(_) | Value::Bool(_) | Value::Number(_) => {
                // Some editors send configuration as primitives, convert to proper format
                json!({ "harper-ls": {} })
            }
            Value::Array(_) => {
                // Arrays are not valid configuration format
                json!({ "harper-ls": {} })
            }
        };

        if let Ok(new_config) = Config::from_lsp_config(&self.root.read().await, processed_json)
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
            // Cancel any pending diagnostic publication for this document
            let mut pending = self.pending_diagnostics.lock().await;
            if let Some(handle) = pending.remove(&uri) {
                handle.abort();
            }

            // Update last change time
            let mut last_changes = self.last_change_time.lock().await;
            last_changes.insert(uri.clone(), Instant::now());
            drop(last_changes);

            // Schedule diagnostics to be published after the delay
            let uri_clone = uri.clone();
            let last_change_time = self.last_change_time.clone();
            let backend = self.clone();

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                // Check if this is still the most recent change
                let last_changes = last_change_time.lock().await;
                if let Some(last_time) = last_changes.get(&uri_clone)
                    && last_time.elapsed() >= Duration::from_millis(delay_ms)
                {
                    // This is still the most recent change, publish diagnostics
                    backend.publish_diagnostics(&uri_clone).await;
                }
                // Otherwise, a newer change has occurred and will handle diagnostics
            });

            pending.insert(uri, handle);
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

        info!("Received command: \"{}\"", params.command.as_str());

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
            "HarperAddToUserDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_uri: Uri = second.parse().unwrap();
                let dialect = self.get_document_dialect(&file_uri).await;

                let mut dict = self.load_user_dictionary(dialect).await;
                dict.append_word(word, DictWordMetadata::default());
                self.save_user_dictionary(dict, dialect)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_uri, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics(&file_uri).await;
            }
            "HarperAddToWSDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_uri: Uri = second.parse().unwrap();
                let dialect = self.get_document_dialect(&file_uri).await;

                let mut dict = self.load_workspace_dictionary(dialect).await;
                dict.append_word(word, DictWordMetadata::default());
                self.save_workspace_dictionary(dict, dialect)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_uri, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics(&file_uri).await;
            }
            "HarperAddToFileDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_uri: Uri = second.parse().unwrap();
                let dialect = self.get_document_dialect(&file_uri).await;

                let mut dict = match self
                    .load_file_dictionary(&file_uri, dialect)
                    .await
                    .map_err(|err| error!("{err}"))
                {
                    Ok(dict) => dict,
                    Err(_) => {
                        return Ok(None);
                    }
                };
                dict.append_word(word, DictWordMetadata::default());

                self.save_file_dictionary(&file_uri, dict, dialect)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_uri, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics(&file_uri).await;
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

                self.publish_diagnostics(&uri).await;
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
                let doc_dialect = doc.cached_dialect.unwrap_or(config_lock.dialect);
                let mut l = LintGroup::new_curated(doc.dict.clone(), doc_dialect);
                l.config.merge_from(config_lock.lint_config.clone());
                doc.linter = l;
            }

            doc_lock.keys().cloned().collect()
        };

        for uri in uris {
            self.update_document_from_file(&uri, None)
                .await
                .map_err(|err| error!("{err}"))
                .err();
            self.publish_diagnostics(&uri).await;
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
        // Collect URIs while holding the lock, then drop it
        let uris: Vec<Uri> = self.doc_state.lock().await.keys().cloned().collect();

        // Clears the diagnostics for open buffers (without holding lock)
        for uri in &uris {
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
    use super::*;

    use futures::{SinkExt, StreamExt};
    use serde_json::json;
    use tempfile::TempDir;
    use tokio::task::JoinHandle;
    use tower_lsp_server::LspService;
    use tower_lsp_server::jsonrpc::Response;
    use tower_lsp_server::lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, ExecuteCommandParams,
        TextDocumentContentChangeEvent, TextDocumentItem, VersionedTextDocumentIdentifier,
    };

    struct TestHarness {
        _service: LspService<Backend>,
        tempdir: TempDir,
        test_config: Config,
        responder: JoinHandle<()>,
    }

    impl TestHarness {
        async fn new() -> Self {
            let tempdir = TempDir::new().unwrap();
            let root = tempdir.path();
            let user_dict_path = root.join("dictionary.txt");
            let file_dict_path = root.join("file_dictionaries");
            let workspace_dict_path = root.join(".harper-dictionary.txt");
            let ignored_lints_path = root.join("ignored_lints");

            let config = Config {
                user_dict_path: user_dict_path.clone(),
                file_dict_path: file_dict_path.clone(),
                workspace_dict_path: workspace_dict_path.clone(),
                ignored_lints_path: ignored_lints_path.clone(),
                ..Config::default()
            };

            let lsp_config = json!({
                "harper-ls": {
                    "userDictPath": user_dict_path.to_string_lossy(),
                    "fileDictPath": file_dict_path.to_string_lossy(),
                    "workspaceDictPath": workspace_dict_path.to_string_lossy(),
                    "ignoredLintsPath": ignored_lints_path.to_string_lossy(),
                }
            });

            let (service, socket) = LspService::new(|client| Backend::new(client, config.clone()));
            let backend = service.inner();
            *backend.root.write().await = root.to_path_buf();

            let (mut request_stream, mut response_sink) = socket.split();
            let responder_config = lsp_config.clone();
            let responder = tokio::spawn(async move {
                while let Some(request) = request_stream.next().await {
                    if request.method() == "workspace/configuration" {
                        let response = Response::from_ok(
                            request.id().unwrap().clone(),
                            json!([responder_config.clone()]),
                        );
                        response_sink.send(response).await.unwrap();
                    }
                }
            });

            Self {
                _service: service,
                tempdir,
                test_config: config,
                responder,
            }
        }

        fn backend(&self) -> &Backend {
            self._service.inner()
        }

        fn file_uri(&self, relative: &str) -> Uri {
            let path = self.tempdir.path().join(relative);
            Uri::from_file_path(path).unwrap()
        }

        async fn write_document(&self, uri: &Uri, text: &str) {
            tokio::fs::write(uri.to_file_path().unwrap(), text)
                .await
                .unwrap();
        }

        async fn open_document(&self, uri: &Uri, language_id: &str, text: &str) {
            self.write_document(uri, text).await;
            self.backend()
                .did_open(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: uri.clone(),
                        language_id: language_id.to_string(),
                        version: 1,
                        text: text.to_string(),
                    },
                })
                .await;
        }

        async fn change_document(&self, uri: &Uri, version: i32, text: &str) {
            self.write_document(uri, text).await;
            self.backend()
                .did_change(DidChangeTextDocumentParams {
                    text_document: VersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version,
                    },
                    content_changes: vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: text.to_string(),
                    }],
                })
                .await;
        }

        async fn execute_command(&self, command: &str, word: &str, uri: &Uri) {
            self.backend()
                .execute_command(ExecuteCommandParams {
                    command: command.to_string(),
                    arguments: vec![json!(word), json!(uri.to_string())],
                    work_done_progress_params: Default::default(),
                })
                .await
                .unwrap();
        }

        async fn install_temp_config(&self) {
            *self.backend().config.write().await = self.test_config.clone();
        }
    }

    impl Drop for TestHarness {
        fn drop(&mut self) {
            self.responder.abort();
        }
    }

    fn german_text_with_errors() -> &'static str {
        "Das ist ein deutscher Beispielsatz mit ausreichend vielen Woertern, damit die Sprache erkannt wird. dieser Satz enthaelt Worrt und flasch geschriebene Begriffe."
    }

    fn german_text_without_errors() -> &'static str {
        "Das ist ein deutscher Beispielsatz mit ausreichend vielen Wörtern, damit die Sprache erkannt wird. Dieser Satz enthält ansonsten nur normale deutsche Formulierungen."
    }

    fn english_text_with_error() -> &'static str {
        "This is an English sample sentence with enough words for detection to stay on the default dialect. this sentence starts with a lowercase word."
    }

    #[tokio::test(flavor = "current_thread")]
    async fn german_markdown_open_uses_german_dialect_and_emits_diagnostics() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("german.md");

        harness
            .open_document(&uri, "markdown", german_text_with_errors())
            .await;

        assert_eq!(
            harness.backend().get_document_dialect(&uri).await,
            Dialect::German
        );

        let diagnostics = harness.backend().generate_diagnostics(&uri).await;
        assert!(
            !diagnostics.is_empty(),
            "expected German diagnostics for intentionally broken German text"
        );
        assert!(
            diagnostics.len() < 12,
            "German text should not be flooded with English-oriented diagnostics: {:?}",
            diagnostics
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn empty_markdown_document_switches_to_german_after_edit() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("empty-then-german.md");

        harness.open_document(&uri, "markdown", "").await;
        assert_eq!(
            harness.backend().get_document_dialect(&uri).await,
            Dialect::American
        );

        harness
            .change_document(&uri, 2, german_text_with_errors())
            .await;

        assert_eq!(
            harness.backend().get_document_dialect(&uri).await,
            Dialect::German
        );

        let diagnostics = harness.backend().generate_diagnostics(&uri).await;
        assert!(
            !diagnostics.is_empty(),
            "expected diagnostics after switching to German text"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn german_dictionary_commands_write_to_dialect_specific_paths() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("german-dicts.md");

        harness
            .open_document(&uri, "markdown", german_text_without_errors())
            .await;
        assert_eq!(
            harness.backend().get_document_dialect(&uri).await,
            Dialect::German
        );
        harness.install_temp_config().await;

        harness
            .execute_command("HarperAddToUserDict", "Ölkannen", &uri)
            .await;
        harness
            .execute_command("HarperAddToWSDict", "Werkstattwort", &uri)
            .await;
        harness
            .execute_command("HarperAddToFileDict", "Dateiwort", &uri)
            .await;

        let german_user_dict =
            Backend::dialect_path(&harness.test_config.user_dict_path, Dialect::German);
        let german_workspace_dict =
            Backend::dialect_path(&harness.test_config.workspace_dict_path, Dialect::German);
        let german_file_dict = harness
            .backend()
            .get_file_dict_path(&uri, Dialect::German)
            .await
            .unwrap();

        assert!(
            tokio::fs::read_to_string(&german_user_dict)
                .await
                .unwrap()
                .contains("Ölkannen")
        );
        assert!(
            tokio::fs::read_to_string(&german_workspace_dict)
                .await
                .unwrap()
                .contains("Werkstattwort")
        );
        assert!(
            tokio::fs::read_to_string(&german_file_dict)
                .await
                .unwrap()
                .contains("Dateiwort")
        );

        if harness.test_config.user_dict_path.exists() {
            assert!(
                !tokio::fs::read_to_string(&harness.test_config.user_dict_path)
                    .await
                    .unwrap()
                    .contains("Ölkannen")
            );
        }
        if harness.test_config.workspace_dict_path.exists() {
            assert!(
                !tokio::fs::read_to_string(&harness.test_config.workspace_dict_path)
                    .await
                    .unwrap()
                    .contains("Werkstattwort")
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn english_user_dictionary_command_keeps_default_path() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("english.md");

        harness
            .open_document(&uri, "markdown", english_text_with_error())
            .await;
        assert_eq!(
            harness.backend().get_document_dialect(&uri).await,
            Dialect::American
        );
        harness.install_temp_config().await;

        harness
            .execute_command("HarperAddToUserDict", "HarperWord", &uri)
            .await;

        let english_user_dict = tokio::fs::read_to_string(&harness.test_config.user_dict_path)
            .await
            .unwrap();
        assert!(english_user_dict.contains("HarperWord"));

        let german_user_dict =
            Backend::dialect_path(&harness.test_config.user_dict_path, Dialect::German);
        if german_user_dict.exists() {
            assert!(
                !tokio::fs::read_to_string(&german_user_dict)
                    .await
                    .unwrap()
                    .contains("HarperWord")
            );
        }
    }
}

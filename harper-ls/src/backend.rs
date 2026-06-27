use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::document_state::DocumentState;
use crate::git_commit_parser::GitCommitParser;
use crate::ignored_lints_io::{load_ignored_lints, save_ignored_lints};
use crate::io_utils::fileify_path;
use anyhow::{Context, Result, anyhow};
use harper_asciidoc::AsciidocParser;
use harper_comments::CommentParser;
use harper_core::language::{
    Language, detect_language, dictionary, new_curated_for_language, parser_for_prose,
};
use harper_core::linting::FlatConfig;
use harper_core::parsers::{CollapseIdentifiers, IsolateEnglish, Parser};
use harper_core::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use harper_core::{DictWordMetadata, Document, IgnoredLints};
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
    Diagnostic, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, ExecuteCommandOptions, ExecuteCommandParams, FileChangeType,
    InitializeParams, InitializeResult, InitializedParams, MessageType, PublishDiagnosticsParams,
    Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Uri,
};
use tower_lsp_server::{Client, LanguageServer, UriExt};
use tracing::{debug, error, info, warn};

/// Return harper-ls version
pub fn ls_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct Backend {
    client: Client,
    root: RwLock<PathBuf>,
    config: RwLock<Config>,
    stats: RwLock<Stats>,
    doc_state: Mutex<HashMap<Uri, DocumentState>>,
}

const MIN_WORDS_FOR_LANGUAGE_DETECTION: usize = 10;

impl Backend {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            root: RwLock::new(".".into()),
            stats: RwLock::new(Stats::new()),
            config: RwLock::new(config),
            doc_state: Mutex::new(HashMap::new()),
        }
    }

    async fn load_file_dictionary(
        &self,
        uri: &Uri,
        language: Language,
    ) -> anyhow::Result<MutableDictionary> {
        // VS Code's unsaved documents have "untitled" scheme
        if uri
            .scheme()
            .is_some_and(|scheme| scheme.eq_lowercase("untitled"))
        {
            return Ok(MutableDictionary::new());
        }

        let path = self
            .get_file_dict_path(uri, language)
            .await
            .context("Unable to get the file path.")?;

        load_dict(path)
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
    /// For non-English languages, files are stored in a language-suffixed directory
    /// (e.g. `file_dictionaries-de/`).
    async fn get_file_dict_path(&self, uri: &Uri, language: Language) -> anyhow::Result<PathBuf> {
        let config = self.config.read().await;
        let suffix = language.family().dict_suffix();

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
        language: Language,
    ) -> Result<()> {
        save_dict(
            self.get_file_dict_path(uri, language)
                .await
                .context("Unable to get the file path.")?,
            dict,
        )
        .await
        .context("Unable to save the dictionary to path.")
    }

    async fn load_user_dictionary(&self, language: Language) -> MutableDictionary {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.user_dict_path, language);

        load_dict(path)
            .await
            .map_err(|err| info!("{err}"))
            .unwrap_or(MutableDictionary::new())
    }

    async fn save_user_dictionary(&self, dict: impl Dictionary, language: Language) -> Result<()> {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.user_dict_path, language);

        save_dict(path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    async fn load_workspace_dictionary(&self, language: Language) -> MutableDictionary {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.workspace_dict_path, language);

        load_dict(path)
            .await
            .map_err(|err| info!("{err}"))
            .unwrap_or(MutableDictionary::new())
    }

    async fn save_workspace_dictionary(
        &self,
        dict: impl Dictionary,
        language: Language,
    ) -> Result<()> {
        let config = self.config.read().await;
        let path = Self::dialect_path(&config.workspace_dict_path, language);

        save_dict(path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    /// Computes a language-aware path from a base path.
    /// For English (default), returns the path unchanged.
    /// For German, inserts a suffix before the extension (or appends it for dirs).
    /// E.g. `dictionary.txt` -> `dictionary-de.txt`,
    ///      `.harper-dictionary.txt` -> `.harper-dictionary-de.txt`.
    fn dialect_path(base: &Path, language: Language) -> PathBuf {
        let suffix = language.family().dict_suffix();
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

    fn save_stats_snapshot(stats_path: PathBuf, stats: Stats) -> Result<()> {
        if let Some(parent) = stats_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut writer = BufWriter::new(
            OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&stats_path)?,
        );
        stats.write(&mut writer)?;
        writer.flush()?;

        Ok(())
    }

    fn queue_stats_save_for_shutdown(&self) {
        let (stats_path, stats_snapshot) = match (self.config.try_read(), self.stats.try_read()) {
            (Ok(config), Ok(stats)) => (config.stats_path.clone(), stats.clone()),
            _ => {
                warn!("Skipping stats save during shutdown because state is busy");
                return;
            }
        };

        if let Err(err) = std::thread::Builder::new()
            .name("harper-ls-shutdown-stats".to_owned())
            .spawn(move || {
                if let Err(err) = Self::save_stats_snapshot(stats_path, stats_snapshot) {
                    error!("Unable to save stats during shutdown: {err}");
                }
            })
        {
            error!("Unable to spawn shutdown stats save thread: {err}");
        }
    }

    async fn generate_global_dictionary(&self, language: Language) -> Result<MergedDictionary> {
        let mut dict = MergedDictionary::new();

        dict.add_dictionary(dictionary(language));
        info!(
            "Loaded {:?} dictionary for language: {:?}",
            language.family(),
            language
        );

        let user_dict = self.load_user_dictionary(language).await;
        dict.add_dictionary(Arc::new(user_dict));
        let ws_dict = self.load_workspace_dictionary(language).await;
        dict.add_dictionary(Arc::new(ws_dict));
        Ok(dict)
    }

    async fn generate_file_dictionary(
        &self,
        uri: &Uri,
        language: Language,
    ) -> Result<MergedDictionary> {
        let (global_dictionary, file_dictionary) = tokio::join!(
            self.generate_global_dictionary(language),
            self.load_file_dictionary(uri, language)
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

    async fn refresh_open_document(&self, uri: &Uri) -> Result<()> {
        let text = {
            let doc_lock = self.doc_state.lock().await;
            doc_lock
                .get(uri)
                .map(|state| state.document.get_full_content().iter().collect::<String>())
        };

        if let Some(text) = text {
            self.update_document(uri, &text, None).await
        } else {
            self.update_document_from_file(uri, None).await
        }
    }

    async fn update_document(
        &self,
        uri: &Uri,
        text: &str,
        language_id: Option<&str>,
    ) -> Result<()> {
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

        let config = self.config.read().await;
        let detected_language: Language = if is_prose {
            let word_count = text.split_whitespace().count();
            info!("Word count: {}", word_count);

            let cached_language = {
                let doc_lock = self.doc_state.lock().await;
                doc_lock.get(uri).and_then(|state| state.cached_language)
            };

            if word_count >= MIN_WORDS_FOR_LANGUAGE_DETECTION {
                // Re-run detection when we have enough content so that e.g. an
                // empty-then-typed document can switch from English to German.
                let dict = FstDictionary::curated();
                let detected = detect_language(text, &dict, config.language);
                debug!(
                    "harper-ls language detect: {:?} for {:?} ({} words)",
                    detected, uri, word_count
                );

                // If the language changed for an existing document, force the linter
                // to be rebuilt by clearing the cached dict.
                if cached_language != Some(detected) {
                    info!(
                        "Language changed from {:?} to {:?}, will rebuild linter",
                        cached_language, detected
                    );
                    let mut doc_lock = self.doc_state.lock().await;
                    if let Some(state) = doc_lock.get_mut(uri) {
                        // Force a dict mismatch so the linter rebuild is triggered.
                        state.dict = Arc::new(MergedDictionary::new());
                    }
                }

                detected
            } else if let Some(cached) = cached_language {
                // Not enough words yet — keep previous detection.
                info!("Using cached language: {:?} ({} words)", cached, word_count);
                cached
            } else {
                info!(
                    "Insufficient content for detection ({} words), using configured language",
                    word_count
                );
                config.language
            }
        } else {
            // For non-prose files (code, etc.) use the configured language.
            config.language
        };

        // Copy necessary configuration to avoid holding lock.
        let (
            lint_config,
            markdown_options,
            isolate_english,
            language, // Configured language (used as fallback)
            max_file_length,
            exclude_patterns,
        ) = (
            config.lint_config.clone(),
            config.markdown_options,
            config.isolate_english,
            config.language,
            config.max_file_length,
            config.exclude_patterns.clone(),
        );

        if !exclude_patterns.is_empty()
            && exclude_patterns.is_match(
                uri.to_file_path()
                    .ok_or_else(|| anyhow!("Unable to convert URI to file path."))?,
            )
        {
            let mut doc_lock = self.doc_state.lock().await;
            doc_lock.remove(uri);
            return Ok(());
        }

        let ignored_lints = self.load_ignored_lints(uri).await.unwrap_or_default();

        let dict = Arc::new(
            self.generate_file_dictionary(uri, detected_language)
                .await
                .context("Unable to generate the file dictionary.")?,
        );

        let mut doc_lock = self.doc_state.lock().await;

        let doc_state = doc_lock.entry(uri.clone()).or_insert_with(|| {
            info!("Constructing new LintGroup for new document.");

            DocumentState {
                ignored_lints,
                linter: {
                    let mut l = new_curated_for_language(dict.clone(), detected_language);
                    l.config.merge_from(lint_config.clone());
                    l
                },
                // Prefer the provided language_id; fall back to what was already stored
                // (effective_language_id already resolved both but we need an owned String).
                language_id: effective_language_id.map(|v| v.to_string()),
                dict: dict.clone(),
                uri: uri.clone(),
                // Store the active language for both prose and non-prose documents so later
                // diagnostics and dictionary code actions stay language-aware.
                cached_language: Some(detected_language),
                ..Default::default()
            }
        });

        // On subsequent updates, also keep the active language in sync.
        doc_state.cached_language = Some(detected_language);

        if doc_state.dict != dict {
            doc_state.dict = dict.clone();
            info!("Constructing new linter because of modified dictionary.");
            let mut l = new_curated_for_language(dict.clone(), detected_language);
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
            language: Language,
        ) -> Result<Box<dyn Parser>> {
            if doc_state.ident_dict != new_dict {
                info!("Constructing new linter because of modified ident dictionary.");
                doc_state.ident_dict = new_dict.clone();

                let mut merged = backend.generate_file_dictionary(uri, language).await?;
                merged.add_dictionary(new_dict);
                let merged = Arc::new(merged);

                doc_state.linter = {
                    let mut l = new_curated_for_language(merged.clone(), language);
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
                            detected_language,
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
                            language,
                        )
                        .await?,
                    )
                } else {
                    Some(Box::new(parser))
                }
            }
            "mail" | "markdown" | "quarto" | "org" | "plaintext" | "text" => {
                parser_for_prose(language_id, detected_language, markdown_options)
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

    /// Returns the detected language for a document, falling back to configured default.
    async fn get_document_language(&self, uri: &Uri) -> Language {
        let doc_lock = self.doc_state.lock().await;
        if let Some(lang) = doc_lock.get(uri).and_then(|state| state.cached_language) {
            return lang;
        }
        self.config.read().await.language
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

    fn shutdown_uris(&self) -> Vec<Uri> {
        match self.doc_state.try_lock() {
            Ok(doc_state) => doc_state.keys().cloned().collect(),
            Err(_) => {
                warn!(
                    "Skipping diagnostic clearing during shutdown because document state is busy"
                );
                Vec::new()
            }
        }
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
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Err(err) = self
            .update_document(
                &params.text_document.uri,
                &params.text_document.text,
                Some(&params.text_document.language_id),
            )
            .await
        {
            error!("{err}");
        }

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(last) = params.content_changes.last() else {
            return;
        };

        if let Err(err) = self
            .update_document(&params.text_document.uri, &last.text, None)
            .await
        {
            error!("{err}")
        }

        self.publish_diagnostics(&params.text_document.uri).await;
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
                let language = self.get_document_language(&file_uri).await;

                let mut dict = self.load_user_dictionary(language).await;
                dict.append_word(word, DictWordMetadata::default());
                self.save_user_dictionary(dict, language)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.refresh_open_document(&file_uri)
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
                let language = self.get_document_language(&file_uri).await;

                let mut dict = self.load_workspace_dictionary(language).await;
                dict.append_word(word, DictWordMetadata::default());
                self.save_workspace_dictionary(dict, language)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.refresh_open_document(&file_uri)
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
                let language = self.get_document_language(&file_uri).await;

                let mut dict = match self
                    .load_file_dictionary(&file_uri, language)
                    .await
                    .map_err(|err| error!("{err}"))
                {
                    Ok(dict) => dict,
                    Err(_) => {
                        return Ok(None);
                    }
                };
                dict.append_word(word, DictWordMetadata::default());

                self.save_file_dictionary(&file_uri, dict, language)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.refresh_open_document(&file_uri)
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

        let (lint_config, default_language) = {
            let config_lock = self.config.read().await;
            (config_lock.lint_config.clone(), config_lock.language)
        };

        let uris: Vec<Uri> = {
            let mut doc_lock = self.doc_state.lock().await;

            for doc in doc_lock.values_mut() {
                info!("Constructing new LintGroup for updated configuration.");
                let doc_language = doc.cached_language.unwrap_or(default_language);
                let mut l = new_curated_for_language(doc.dict.clone(), doc_language);
                l.config.merge_from(lint_config.clone());
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

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // Explicitly implemented so clients like Helix don't log
        // "textDocument/didSave notification not implemented".
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
        // Avoid blocking shutdown on long-running document updates.
        // If the lock is contended, skip diagnostic clearing and continue shutdown.
        let uris = self.shutdown_uris();

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

        self.queue_stats_save_for_shutdown();

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
            let stats_path = root.join("stats.txt");

            let config = Config {
                user_dict_path: user_dict_path.clone(),
                file_dict_path: file_dict_path.clone(),
                workspace_dict_path: workspace_dict_path.clone(),
                ignored_lints_path: ignored_lints_path.clone(),
                stats_path: stats_path.clone(),
                ..Config::default()
            };

            let lsp_config = json!({
                "harper-ls": {
                    "userDictPath": user_dict_path.to_string_lossy(),
                    "fileDictPath": file_dict_path.to_string_lossy(),
                    "workspaceDictPath": workspace_dict_path.to_string_lossy(),
                    "ignoredLintsPath": ignored_lints_path.to_string_lossy(),
                    "statsPath": stats_path.to_string_lossy(),
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

    fn german_text_with_workspace_word() -> &'static str {
        "Das ist ein deutscher Beispielsatz mit ausreichend vielen Wörtern, damit die Sprache erkannt wird. Dieses Zqxjwort steht nur im Editorpuffer."
    }

    fn german_text_on_disk_after_unsaved_edit() -> &'static str {
        "Das ist weiterhin ein deutscher Beispielsatz mit ausreichend vielen Wörtern, aber ohne das neue Token im gespeicherten Zustand."
    }

    #[tokio::test(flavor = "current_thread")]
    async fn german_markdown_open_uses_german_dialect_and_emits_diagnostics() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("german.md");

        harness
            .open_document(&uri, "markdown", german_text_with_errors())
            .await;

        assert_eq!(
            harness.backend().get_document_language(&uri).await,
            harper_core::language::parse_language("de").unwrap()
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
            harness.backend().get_document_language(&uri).await,
            Language::default()
        );

        harness
            .change_document(&uri, 2, german_text_with_errors())
            .await;

        assert_eq!(
            harness.backend().get_document_language(&uri).await,
            harper_core::language::parse_language("de").unwrap()
        );

        let diagnostics = harness.backend().generate_diagnostics(&uri).await;
        assert!(
            !diagnostics.is_empty(),
            "expected diagnostics after switching to German text"
        );
        assert!(
            diagnostics.iter().any(|d| {
                d.code.as_ref().is_some_and(|code| {
                    matches!(
                        code,
                        tower_lsp_server::lsp_types::NumberOrString::String(tag)
                            if tag.contains("GermanSpellCheck")
                    )
                })
            }),
            "expected German spellcheck diagnostics after language switch: {:?}",
            diagnostics
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn german_dictionary_commands_write_to_language_specific_paths() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("german-dicts.md");

        let german_lang = harper_core::language::parse_language("de").unwrap();

        harness
            .open_document(&uri, "markdown", german_text_without_errors())
            .await;
        assert_eq!(
            harness.backend().get_document_language(&uri).await,
            german_lang
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
            Backend::dialect_path(&harness.test_config.user_dict_path, german_lang);
        let german_workspace_dict =
            Backend::dialect_path(&harness.test_config.workspace_dict_path, german_lang);
        let german_file_dict = harness
            .backend()
            .get_file_dict_path(&uri, german_lang)
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
    async fn workspace_dictionary_command_uses_open_buffer_text_for_refresh() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("german-unsaved-buffer.md");

        let german_lang = harper_core::language::parse_language("de").unwrap();

        harness
            .open_document(&uri, "markdown", german_text_with_workspace_word())
            .await;
        assert_eq!(
            harness.backend().get_document_language(&uri).await,
            german_lang
        );
        harness.install_temp_config().await;

        let before = harness.backend().generate_diagnostics(&uri).await;
        assert!(
            before
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Zqxjwort")),
            "expected an initial spelling diagnostic for the unsaved buffer word: {before:?}"
        );

        harness
            .write_document(&uri, german_text_on_disk_after_unsaved_edit())
            .await;

        harness
            .execute_command("HarperAddToWSDict", "Zqxjwort", &uri)
            .await;

        let current_text = {
            let doc_lock = harness.backend().doc_state.lock().await;
            let state = doc_lock.get(&uri).expect("document should stay open");
            state.document.get_full_content().iter().collect::<String>()
        };
        assert!(
            current_text.contains("Zqxjwort"),
            "refresh should preserve the in-memory editor buffer, got: {current_text}"
        );

        let after = harness.backend().generate_diagnostics(&uri).await;
        assert!(
            after
                .iter()
                .all(|diagnostic| !diagnostic.message.contains("Zqxjwort")),
            "workspace dictionary addition should clear the buffer spelling lint: {after:?}"
        );

        harness
            .backend()
            .update_document(&uri, german_text_with_workspace_word(), None)
            .await
            .unwrap();
        let reopened = harness.backend().generate_diagnostics(&uri).await;
        assert!(
            reopened
                .iter()
                .all(|diagnostic| !diagnostic.message.contains("Zqxjwort")),
            "workspace dictionary addition should persist across a reopen-like update: {reopened:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn english_user_dictionary_command_keeps_default_path() {
        let harness = TestHarness::new().await;
        let uri = harness.file_uri("english.md");

        harness
            .open_document(&uri, "markdown", english_text_with_error())
            .await;
        assert_eq!(
            harness.backend().get_document_language(&uri).await,
            Language::default()
        );
        harness.install_temp_config().await;

        harness
            .execute_command("HarperAddToUserDict", "HarperWord", &uri)
            .await;

        let english_user_dict = tokio::fs::read_to_string(&harness.test_config.user_dict_path)
            .await
            .unwrap();
        assert!(english_user_dict.contains("HarperWord"));

        let german_lang = harper_core::language::parse_language("de").unwrap();
        let german_user_dict =
            Backend::dialect_path(&harness.test_config.user_dict_path, german_lang);
        if german_user_dict.exists() {
            assert!(
                !tokio::fs::read_to_string(&german_user_dict)
                    .await
                    .unwrap()
                    .contains("HarperWord")
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_persists_stats_file() {
        let harness = TestHarness::new().await;
        harness.install_temp_config().await;

        harness.backend().shutdown().await.unwrap();
        for _ in 0..20 {
            if harness.test_config.stats_path.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        assert!(
            harness.test_config.stats_path.exists(),
            "shutdown should persist stats file"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_uris_returns_empty_when_doc_state_lock_is_busy() {
        let harness = TestHarness::new().await;
        let _doc_state_guard = harness.backend().doc_state.lock().await;

        assert!(
            harness.backend().shutdown_uris().is_empty(),
            "shutdown should skip URI collection when doc_state is contended"
        );
    }
}

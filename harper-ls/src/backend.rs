use std::collections::HashMap;
use std::path::{Component, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{CollapseIdentifiers, IsolateEnglish, Markdown, Parser, PlainEnglish};
use harper_core::{
    Dictionary, Document, FstDictionary, FullDictionary, MergedDictionary, Token, TokenKind,
    WordMetadata,
};
use harper_html::HtmlParser;
use harper_literate_haskell::LiterateHaskellParser;
use harper_typst::Typst;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::jsonrpc::Result as JsonResult;
use tower_lsp::lsp_types::notification::PublishDiagnostics;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    Command, ConfigurationItem, Diagnostic, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidChangeWatchedFilesRegistrationOptions, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, ExecuteCommandOptions,
    ExecuteCommandParams, FileChangeType, FileSystemWatcher, GlobPattern, InitializeParams,
    InitializeResult, InitializedParams, MessageType, PublishDiagnosticsParams, Range,
    Registration, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url, WatchKind,
};
use tower_lsp::{Client, LanguageServer};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::diagnostics::{lint_to_code_actions, lints_to_diagnostics};
use crate::dictionary_io::{load_dict, save_dict};
use crate::document_state::DocumentState;
use crate::git_commit_parser::GitCommitParser;
use crate::pos_conv::range_to_span;

pub struct Backend {
    client: Client,
    config: RwLock<Config>,
    doc_state: Mutex<HashMap<Url, DocumentState>>,
}

impl Backend {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            config: RwLock::new(config),
            doc_state: Mutex::new(HashMap::new()),
        }
    }

    /// Rewrites a path to a filename using the same conventions as
    /// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
    fn file_dict_name(url: &Url) -> anyhow::Result<PathBuf> {
        let mut rewritten = String::new();

        // We assume all URLs are local files and have a base.
        for seg in url
            .to_file_path()
            .map_err(|_| anyhow!("Unable to convert URL to file path."))?
            .components()
        {
            if !matches!(seg, Component::RootDir) {
                rewritten.push_str(&seg.as_os_str().to_string_lossy());
                rewritten.push('%');
            }
        }

        Ok(rewritten.into())
    }

    /// Get the location of the file's specific dictionary
    async fn get_file_dict_path(&self, url: &Url) -> anyhow::Result<PathBuf> {
        let config = self.config.read().await;

        Ok(config.file_dict_path.join(Self::file_dict_name(url)?))
    }

    /// Load a specific file's dictionary
    async fn load_file_dictionary(&self, url: &Url) -> anyhow::Result<FullDictionary> {
        let path = self
            .get_file_dict_path(url)
            .await
            .context("Unable to get the file path.")?;

        load_dict(path)
            .await
            .map_err(|err| debug!("Attempting to open file dictionary: {err}"))
            .or(Ok(FullDictionary::new()))
    }

    async fn save_file_dictionary(&self, url: &Url, dict: impl Dictionary) -> Result<()> {
        save_dict(
            self.get_file_dict_path(url)
                .await
                .context("Unable to get the file path.")?,
            dict,
        )
        .await
        .context("Unable to save the dictionary to path.")
    }

    async fn load_user_dictionary(&self) -> FullDictionary {
        let config = self.config.read().await;

        load_dict(&config.user_dict_path)
            .await
            .map_err(|err| info!("{err}"))
            .unwrap_or(FullDictionary::new())
    }

    async fn save_user_dictionary(&self, dict: impl Dictionary) -> Result<()> {
        let config = self.config.read().await;

        save_dict(&config.user_dict_path, dict)
            .await
            .map_err(|err| anyhow!("Unable to save the dictionary to file: {err}"))
    }

    async fn generate_global_dictionary(&self) -> Result<MergedDictionary> {
        let mut dict = MergedDictionary::new();
        dict.add_dictionary(FstDictionary::curated());
        let user_dict = self.load_user_dictionary().await;
        dict.add_dictionary(Arc::new(user_dict));
        Ok(dict)
    }

    async fn generate_file_dictionary(&self, url: &Url) -> Result<MergedDictionary> {
        let (global_dictionary, file_dictionary) = tokio::join!(
            self.generate_global_dictionary(),
            self.load_file_dictionary(url)
        );

        let mut global_dictionary =
            global_dictionary.context("Unable to load the global dictionary.")?;
        global_dictionary.add_dictionary(Arc::new(
            file_dictionary.context("Unable to load the file dictionary.")?,
        ));

        Ok(global_dictionary)
    }

    async fn update_document_from_file(&self, url: &Url, language_id: Option<&str>) -> Result<()> {
        let content = tokio::fs::read_to_string(
            url.to_file_path()
                .map_err(|_| anyhow!("Unable to convert URL to file path."))?,
        )
        .await
        .with_context(|| format!("Unable to read from file {:?}", url))?;

        self.update_document(url, &content, language_id).await
    }

    async fn update_document(
        &self,
        url: &Url,
        text: &str,
        language_id: Option<&str>,
    ) -> Result<()> {
        self.pull_config().await;

        let mut doc_lock = self.doc_state.lock().await;
        let config_lock = self.config.read().await;

        let dict = Arc::new(
            self.generate_file_dictionary(url)
                .await
                .context("Unable to generate the file dictionary.")?,
        );

        let doc_state = doc_lock.entry(url.clone()).or_insert(DocumentState {
            linter: LintGroup::new(config_lock.lint_config, dict.clone()),
            language_id: language_id.map(|v| v.to_string()),
            dict: dict.clone(),
            ..Default::default()
        });

        if doc_state.dict != dict {
            doc_state.dict = dict.clone();
            doc_state.linter = LintGroup::new(config_lock.lint_config, dict.clone());
        }

        let Some(language_id) = &doc_state.language_id else {
            doc_lock.remove(url);
            return Ok(());
        };

        async fn use_ident_dict<'a>(
            backend: &'a Backend,
            new_dict: Arc<FullDictionary>,
            parser: impl Parser + 'static,
            url: &'a Url,
            doc_state: &'a mut DocumentState,
            config_lock: tokio::sync::RwLockReadGuard<'a, Config>,
        ) -> Result<Box<dyn Parser>> {
            if doc_state.ident_dict != new_dict {
                doc_state.ident_dict = new_dict.clone();

                let mut merged = backend.generate_file_dictionary(url).await?;
                merged.add_dictionary(new_dict);
                let merged = Arc::new(merged);

                doc_state.linter = LintGroup::new(config_lock.lint_config, merged.clone());
                doc_state.dict = merged.clone();
            }

            Ok(Box::new(CollapseIdentifiers::new(
                Box::new(parser),
                Box::new(doc_state.dict.clone()),
            )))
        }

        let markdown_options = self.config.read().await.markdown_options;
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
                            url,
                            doc_state,
                            config_lock,
                        )
                        .await?,
                    )
                } else {
                    Some(Box::new(ts_parser))
                }
            }
            "literate haskell" | "lhaskell" => {
                let parser = LiterateHaskellParser::new_markdown(markdown_options);

                if let Some(new_dict) =
                    parser.create_ident_dict(&Arc::new(source), markdown_options)
                {
                    Some(
                        use_ident_dict(
                            self,
                            Arc::new(new_dict),
                            parser,
                            url,
                            doc_state,
                            config_lock,
                        )
                        .await?,
                    )
                } else {
                    Some(Box::new(parser))
                }
            }
            "markdown" => Some(Box::new(Markdown::new(markdown_options))),
            "git-commit" | "gitcommit" => {
                Some(Box::new(GitCommitParser::new_markdown(markdown_options)))
            }
            "html" => Some(Box::new(HtmlParser::default())),
            "mail" | "plaintext" => Some(Box::new(PlainEnglish)),
            "typst" => Some(Box::new(Typst)),
            _ => None,
        };

        match parser {
            None => {
                doc_lock.remove(url);
            }
            Some(mut parser) => {
                if self.config.read().await.isolate_english {
                    parser = Box::new(IsolateEnglish::new(parser, doc_state.dict.clone()));
                }

                doc_state.document = Document::new(text, &parser, &doc_state.dict);
            }
        }

        Ok(())
    }

    async fn generate_code_actions(
        &self,
        url: &Url,
        range: Range,
    ) -> JsonResult<Vec<CodeActionOrCommand>> {
        let (config, mut doc_states) = tokio::join!(self.config.read(), self.doc_state.lock());
        let Some(doc_state) = doc_states.get_mut(url) else {
            return Ok(Vec::new());
        };

        let mut lints = doc_state.linter.lint(&doc_state.document);
        lints.sort_by_key(|l| l.priority);

        let source_chars = doc_state.document.get_full_content();

        // Find lints whole span overlaps with range
        let span = range_to_span(source_chars, range).with_len(1);

        let mut actions: Vec<CodeActionOrCommand> = lints
            .into_iter()
            .filter(|lint| lint.span.overlaps_with(span))
            .flat_map(|lint| {
                lint_to_code_actions(&lint, url, source_chars, &config.code_action_config)
            })
            .collect();

        if let Some(Token {
            kind: TokenKind::Url,
            span,
            ..
        }) = doc_state.document.get_token_at_char_index(span.start)
        {
            actions.push(CodeActionOrCommand::Command(Command::new(
                "Open URL".to_string(),
                "HarperOpen".to_string(),
                Some(vec![doc_state.document.get_span_content_str(span).into()]),
            )))
        }

        Ok(actions)
    }

    async fn generate_diagnostics(&self, url: &Url) -> Vec<Diagnostic> {
        let mut doc_states = self.doc_state.lock().await;
        let Some(doc_state) = doc_states.get_mut(url) else {
            return Vec::new();
        };

        let lints = doc_state.linter.lint(&doc_state.document);
        let config = self.config.read().await;

        lints_to_diagnostics(
            doc_state.document.get_full_content(),
            &lints,
            config.diagnostic_severity,
        )
    }

    async fn publish_diagnostics(&self, url: &Url) {
        let diagnostics = self.generate_diagnostics(url).await;

        let result = PublishDiagnosticsParams {
            uri: url.clone(),
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
        if let Ok(new_config) = Config::from_lsp_config(json_obj).map_err(|err| error!("{err}")) {
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
            .unwrap();

        if let Some(first) = new_config.pop() {
            self.update_config_from_obj(first).await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> JsonResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "HarperAddToUserDict".to_owned(),
                        "HarperAddToFileDict".to_owned(),
                        "HarperOpen".to_owned(),
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

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.update_document_from_file(&params.text_document.uri, None)
            .await
            .map_err(|err| error!("{err}"))
            .err();

        self.publish_diagnostics(&params.text_document.uri).await;
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

        if let Err(err) = self
            .update_document(&params.text_document.uri, &last.text, None)
            .await
        {
            error!("{err}")
        }

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {}

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let mut doc_lock = self.doc_state.lock().await;
        let mut urls_to_clear = Vec::new();

        for change in &params.changes {
            if change.typ != FileChangeType::DELETED {
                continue;
            }

            doc_lock.retain(|url, _| {
                // `change.uri` could be a directory so use `starts_with` instead of `==`.
                let to_remove = url.as_str().starts_with(change.uri.as_str());

                if to_remove {
                    urls_to_clear.push(url.clone());
                }

                !to_remove
            });
        }

        for url in &urls_to_clear {
            self.client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: url.clone(),
                    diagnostics: vec![],
                    version: None,
                })
                .await;
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> JsonResult<Option<Value>> {
        let mut string_args = params
            .arguments
            .into_iter()
            .map(|v| serde_json::from_value::<String>(v).unwrap());

        let Some(first) = string_args.next() else {
            return Ok(None);
        };

        info!("Received command: \"{}\"", params.command.as_str());

        match params.command.as_str() {
            "HarperAddToUserDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_url = second.parse().unwrap();

                let mut dict = self.load_user_dictionary().await;
                dict.append_word(word, WordMetadata::default());
                self.save_user_dictionary(dict)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_url, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics(&file_url).await;
            }
            "HarperAddToFileDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_url = second.parse().unwrap();

                let mut dict = match self
                    .load_file_dictionary(&file_url)
                    .await
                    .map_err(|err| error!("{err}"))
                {
                    Ok(dict) => dict,
                    Err(_) => {
                        return Ok(None);
                    }
                };
                dict.append_word(word, WordMetadata::default());

                self.save_file_dictionary(&file_url, dict)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.update_document_from_file(&file_url, None)
                    .await
                    .map_err(|err| error!("{err}"))
                    .err();
                self.publish_diagnostics(&file_url).await;
            }
            "HarperOpen" => match open::that(&first) {
                Ok(()) => {
                    let message = format!(r#"Opened "{}""#, first);

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
            _ => (),
        }

        Ok(None)
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.update_config_from_obj(params.settings).await;

        let urls: Vec<Url> = {
            let mut doc_lock = self.doc_state.lock().await;
            let config_lock = self.config.read().await;

            for doc in doc_lock.values_mut() {
                doc.linter = LintGroup::new(config_lock.lint_config, doc.dict.clone());
            }

            doc_lock.keys().cloned().collect()
        };

        for url in urls {
            self.update_document_from_file(&url, None)
                .await
                .map_err(|err| error!("{err}"))
                .err();
            self.publish_diagnostics(&url).await;
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
        for url in doc_states.keys() {
            let result = PublishDiagnosticsParams {
                uri: url.clone(),
                diagnostics: vec![],
                version: None,
            };

            self.client
                .send_notification::<PublishDiagnostics>(result)
                .await;
        }

        Ok(())
    }
}

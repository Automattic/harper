use crate::config::Config;
use harper_core::{DictWordMetadata, spell::Dictionary};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};
use tokio::sync::Mutex;

use super::error::ProtocolError;
use super::framing::write_message;
use super::message::{Request, Response};

/// Tauri-side protocol endpoint that owns shared state and responds to highlighter requests.
pub struct Server<R, W> {
    reader: BufReader<R>,
    writer: W,
    config: Arc<Mutex<Config>>,
}

impl<R, W> Server<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn new(reader: R, writer: W, config: Arc<Mutex<Config>>) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
            config,
        }
    }

    pub async fn receive_request(&mut self) -> Result<Option<Request>, ProtocolError> {
        let mut line = String::new();
        if self.reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }

        let request = serde_json::from_str(&line)?;
        let response = self.handle_request(&request).await;
        write_message(&mut self.writer, &response).await?;

        Ok(Some(request))
    }

    async fn handle_request(&self, request: &Request) -> Response {
        match request {
            Request::GetLintConfig => Response::GetLintConfig {
                config: self.config.lock().await.lint_config.clone(),
            },
            Request::GetDictionary => Response::GetDictionary {
                words: self
                    .config
                    .lock()
                    .await
                    .mutable_dictionary
                    .words_iter()
                    .map(|word| word.iter().collect())
                    .collect(),
            },
            Request::GetDialect => Response::GetDialect {
                dialect: self.config.lock().await.dialect,
            },
            Request::GetIgnoredLints => Response::GetIgnoredLints {
                ignored_lints: self.config.lock().await.ignored_lints.clone(),
            },
            Request::GetAllowedBundleIdentifiers => Response::GetAllowedBundleIdentifiers {
                bundle_identifiers: self.config.lock().await.allowed_bundle_identifiers.clone(),
            },
            Request::SetLintConfig { config } => {
                let mut stored_config = self.config.lock().await;
                stored_config.lint_config = config.clone();

                if let Err(error) = stored_config.save_to_system().await {
                    eprintln!("failed to save config: {error}");
                }

                Response::Ack
            }
            Request::IgnoreLint { ignored_lints } => {
                let mut config = self.config.lock().await;
                config.ignored_lints.append(ignored_lints.clone());

                if let Err(error) = config.save_to_system().await {
                    eprintln!("failed to save config: {error}");
                }

                Response::Ack
            }
            Request::AddToDictionary { word } => {
                let mut config = self.config.lock().await;
                config
                    .mutable_dictionary
                    .append_word_str(word, DictWordMetadata::default());

                if let Err(error) = config.save_to_system().await {
                    eprintln!("failed to save config: {error}");
                }

                Response::Ack
            }
            Request::AddAllowedBundleIdentifier { bundle_identifier } => {
                let mut config = self.config.lock().await;
                config.add_allowed_bundle_identifier(bundle_identifier.clone());

                if let Err(error) = config.save_to_system().await {
                    eprintln!("failed to save config: {error}");
                }

                Response::Ack
            }
            Request::RemoveAllowedBundleIdentifier { bundle_identifier } => {
                let mut config = self.config.lock().await;
                config.remove_allowed_bundle_identifier(bundle_identifier);

                if let Err(error) = config.save_to_system().await {
                    eprintln!("failed to save config: {error}");
                }

                Response::Ack
            }
        }
    }
}

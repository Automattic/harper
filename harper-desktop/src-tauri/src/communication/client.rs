use harper_core::{
    DictWordMetadata, IgnoredLints, Language, linting::FlatConfig, spell::MutableDictionary,
};
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader, Stdin, Stdout},
    time::timeout,
};

use super::error::ProtocolError;
use super::framing::write_message;
use super::message::{Request, Response};

/// Highlighter-side protocol endpoint for requesting state from the Tauri server over async I/O.
pub struct Client<R, W> {
    reader: BufReader<R>,
    writer: W,
}

impl<R, W> Client<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
        }
    }

    pub async fn get_lint_config(&mut self) -> Result<FlatConfig, ProtocolError> {
        match self.send_request(Request::GetLintConfig).await? {
            Response::GetLintConfig { config } => Ok(config),
            _ => Err(ProtocolError::UnexpectedResponse {
                expected: "GetLintConfig",
            }),
        }
    }

    pub async fn get_dictionary(&mut self) -> Result<MutableDictionary, ProtocolError> {
        match self.send_request(Request::GetDictionary).await? {
            Response::GetDictionary { words } => {
                let mut dictionary = MutableDictionary::new();
                dictionary.extend_words(words.into_iter().map(|word| {
                    (
                        word.chars().collect::<Vec<_>>(),
                        DictWordMetadata::default(),
                    )
                }));

                Ok(dictionary)
            }
            _ => Err(ProtocolError::UnexpectedResponse {
                expected: "GetDictionary",
            }),
        }
    }

    pub async fn get_dialect(&mut self) -> Result<Language, ProtocolError> {
        match self.send_request(Request::GetDialect).await? {
            Response::GetDialect { dialect } => Ok(dialect),
            _ => Err(ProtocolError::UnexpectedResponse {
                expected: "GetDialect",
            }),
        }
    }

    pub async fn get_debounce_ms(&mut self) -> Result<u64, ProtocolError> {
        match self.send_request(Request::GetDebounceMs).await? {
            Response::GetDebounceMs { debounce_ms } => Ok(debounce_ms),
            _ => Err(ProtocolError::UnexpectedResponse {
                expected: "GetDebounceMs",
            }),
        }
    }

    pub async fn get_ignored_lints(&mut self) -> Result<IgnoredLints, ProtocolError> {
        match self.send_request(Request::GetIgnoredLints).await? {
            Response::GetIgnoredLints { ignored_lints } => Ok(ignored_lints),
            _ => Err(ProtocolError::UnexpectedResponse {
                expected: "GetIgnoredLints",
            }),
        }
    }

    pub async fn get_integrations(
        &mut self,
    ) -> Result<Vec<crate::config::Integration>, ProtocolError> {
        match self.send_request(Request::GetIntegrations).await? {
            Response::GetIntegrations { integrations } => Ok(integrations),
            _ => Err(ProtocolError::UnexpectedResponse {
                expected: "GetIntegrations",
            }),
        }
    }

    pub async fn ignore_lint(&mut self, ignored_lints: &IgnoredLints) -> Result<(), ProtocolError> {
        self.send_ack_request(Request::IgnoreLint {
            ignored_lints: ignored_lints.clone(),
        })
        .await
    }

    pub async fn set_lint_config(&mut self, config: &FlatConfig) -> Result<(), ProtocolError> {
        self.send_ack_request(Request::SetLintConfig {
            config: config.clone(),
        })
        .await
    }

    pub async fn disable_rule(&mut self, rule_name: &str) -> Result<FlatConfig, ProtocolError> {
        let mut config = self.get_lint_config().await?;
        config.set_rule_enabled(rule_name, false);
        self.set_lint_config(&config).await?;

        Ok(config)
    }

    pub async fn add_to_dictionary(&mut self, word: &str) -> Result<(), ProtocolError> {
        self.send_ack_request(Request::AddToDictionary {
            word: word.to_string(),
        })
        .await
    }

    pub async fn add_integration(&mut self, bundle_id: &str) -> Result<(), ProtocolError> {
        self.send_ack_request(Request::AddIntegration {
            bundle_id: bundle_id.to_string(),
        })
        .await
    }

    pub async fn remove_integration(&mut self, bundle_id: &str) -> Result<(), ProtocolError> {
        self.send_ack_request(Request::RemoveIntegration {
            bundle_id: bundle_id.to_string(),
        })
        .await
    }

    pub async fn set_integration_enabled(
        &mut self,
        bundle_id: &str,
        enabled: bool,
    ) -> Result<(), ProtocolError> {
        self.send_ack_request(Request::SetIntegrationEnabled {
            bundle_id: bundle_id.to_string(),
            enabled,
        })
        .await
    }

    async fn send_ack_request(&mut self, request: Request) -> Result<(), ProtocolError> {
        match self.send_request(request).await? {
            Response::Ack => Ok(()),
            _ => Err(ProtocolError::UnexpectedResponse { expected: "Ack" }),
        }
    }

    async fn send_request(&mut self, request: Request) -> Result<Response, ProtocolError> {
        write_message(&mut self.writer, &request).await?;

        let mut line = String::new();
        let response = timeout(Duration::from_secs(5), self.reader.read_line(&mut line)).await;
        let bytes_read = match response {
            Ok(bytes_read) => bytes_read?,
            Err(_) => {
                eprintln!("parent process did not respond within 5 seconds; exiting highlighter");
                std::process::exit(1);
            }
        };

        if bytes_read == 0 {
            return Err(ProtocolError::UnexpectedEof);
        }

        Ok(serde_json::from_str(&line)?)
    }
}

impl Client<Stdin, Stdout> {
    pub fn current_process() -> Self {
        Self::new(tokio::io::stdin(), tokio::io::stdout())
    }
}

use harper_core::{IgnoredLints, linting::FlatConfig};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader, Stdin, Stdout};

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
            Response::Ack => Err(ProtocolError::UnexpectedResponse {
                expected: "GetLintConfig",
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

    async fn send_ack_request(&mut self, request: Request) -> Result<(), ProtocolError> {
        match self.send_request(request).await? {
            Response::Ack => Ok(()),
            Response::GetLintConfig { .. } => {
                Err(ProtocolError::UnexpectedResponse { expected: "Ack" })
            }
        }
    }

    async fn send_request(&mut self, request: Request) -> Result<Response, ProtocolError> {
        write_message(&mut self.writer, &request).await?;

        let mut line = String::new();
        if self.reader.read_line(&mut line).await? == 0 {
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

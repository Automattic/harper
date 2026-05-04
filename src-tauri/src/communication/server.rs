use harper_core::linting::FlatConfig;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use super::error::ProtocolError;
use super::framing::write_message;
use super::message::{Request, Response};

/// Tauri-side protocol endpoint that owns shared state and responds to highlighter requests.
pub struct Server<R, W> {
    reader: BufReader<R>,
    writer: W,
    lint_config: FlatConfig,
}

impl<R, W> Server<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn new(reader: R, writer: W) -> Self {
        Self::with_lint_config(reader, writer, FlatConfig::new_curated())
    }

    pub fn with_lint_config(reader: R, writer: W, lint_config: FlatConfig) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
            lint_config,
        }
    }

    pub async fn receive_request(&mut self) -> Result<Option<Request>, ProtocolError> {
        let mut line = String::new();
        if self.reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }

        let request = serde_json::from_str(&line)?;
        let response = self.handle_request(&request);
        write_message(&mut self.writer, &response).await?;

        Ok(Some(request))
    }

    fn handle_request(&self, request: &Request) -> Response {
        match request {
            Request::GetLintConfig => Response::GetLintConfig {
                config: self.lint_config.clone(),
            },
        }
    }
}

use crate::config::Config;
use harper_core::DictWordMetadata;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

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
        let response = self.handle_request(&request);
        write_message(&mut self.writer, &response).await?;

        Ok(Some(request))
    }

    fn handle_request(&self, request: &Request) -> Response {
        match request {
            Request::GetLintConfig => Response::GetLintConfig {
                config: self
                    .config
                    .lock()
                    .expect("config mutex poisoned")
                    .lint_config
                    .clone(),
            },
            Request::IgnoreLint { ignored_lints } => {
                self.config
                    .lock()
                    .expect("config mutex poisoned")
                    .ignored_lints
                    .append(ignored_lints.clone());

                Response::Ack
            }
            Request::AddToDictionary { word } => {
                self.config
                    .lock()
                    .expect("config mutex poisoned")
                    .mutable_dictionary
                    .append_word_str(word, DictWordMetadata::default());

                Response::Ack
            }
        }
    }
}

mod client;
mod error;
mod framing;
mod message;
mod server;

pub use client::Client;
pub use error::ProtocolError;
pub use message::{Request, Response};
pub use server::Server;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use harper_core::{Document, IgnoredLints, linting::FlatConfig, linting::Lint};
    use std::sync::{Arc, Mutex};
    use tokio::io::{duplex, empty, sink};

    #[tokio::test]
    async fn client_receives_lint_config_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let expected = FlatConfig::new_curated();
        let mut config = Config::new();
        config.lint_config = expected.clone();
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(server_request_reader, server_response_writer, config);

        let (config, request) = tokio::join!(client.get_lint_config(), server.receive_request());

        assert_eq!(config.unwrap(), expected);
        assert!(matches!(request.unwrap(), Some(Request::GetLintConfig)));
    }

    #[tokio::test]
    async fn client_can_merge_ignored_lints_on_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let document = Document::new_markdown_default_curated("A test document.");
        let lint = Lint::default();
        let mut ignored_lints = IgnoredLints::new();
        ignored_lints.ignore_lint(&lint, &document);
        let config = Arc::new(Mutex::new(Config::new()));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );

        let (ack, request) =
            tokio::join!(client.ignore_lint(&ignored_lints), server.receive_request());

        assert!(ack.is_ok());
        assert!(matches!(request.unwrap(), Some(Request::IgnoreLint { .. })));
        assert!(
            config
                .lock()
                .unwrap()
                .ignored_lints
                .is_ignored(&lint, &document)
        );
    }

    #[tokio::test]
    async fn server_returns_none_on_eof() {
        let config = Arc::new(Mutex::new(Config::new()));
        let mut server = Server::new(empty(), sink(), config);

        assert!(server.receive_request().await.unwrap().is_none());
    }
}

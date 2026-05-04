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
    use harper_core::linting::FlatConfig;
    use tokio::io::{duplex, empty, sink};

    #[tokio::test]
    async fn client_receives_lint_config_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let expected = FlatConfig::new_curated();
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::with_lint_config(
            server_request_reader,
            server_response_writer,
            expected.clone(),
        );

        let (config, request) = tokio::join!(client.get_lint_config(), server.receive_request());

        assert_eq!(config.unwrap(), expected);
        assert_eq!(request.unwrap(), Some(Request::GetLintConfig));
    }

    #[tokio::test]
    async fn server_returns_none_on_eof() {
        let mut server = Server::new(empty(), sink());

        assert_eq!(server.receive_request().await.unwrap(), None);
    }
}

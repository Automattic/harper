//! Contains the protocol for the communication between the highlighter [`Client`] and the main
//! process [`Server`].

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
    use harper_core::{
        DictWordMetadata, Document, EnglishDialect, IgnoredLints, Language, linting::FlatConfig,
        linting::Lint, spell::Dictionary,
    };
    use std::sync::Arc;
    use tokio::io::{duplex, empty, sink};
    use tokio::sync::Mutex;

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
    async fn client_receives_dictionary_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let mut config = Config::new();
        config
            .mutable_dictionary
            .append_word_str("blorple", DictWordMetadata::default());
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(server_request_reader, server_response_writer, config);

        let (dictionary, request) = tokio::join!(client.get_dictionary(), server.receive_request());

        assert!(dictionary.unwrap().contains_word_str("blorple"));
        assert!(matches!(request.unwrap(), Some(Request::GetDictionary)));
    }

    #[tokio::test]
    async fn client_receives_dialect_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let mut config = Config::new();
        config.dialect = Language::English(EnglishDialect::British);
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(server_request_reader, server_response_writer, config);

        let (dialect, request) = tokio::join!(client.get_dialect(), server.receive_request());

        assert_eq!(dialect.unwrap(), Language::English(EnglishDialect::British));
        assert!(matches!(request.unwrap(), Some(Request::GetDialect)));
    }

    #[tokio::test]
    async fn client_receives_debounce_ms_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let mut config = Config::new();
        config.debounce_ms = 250;
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(server_request_reader, server_response_writer, config);

        let (debounce_ms, request) =
            tokio::join!(client.get_debounce_ms(), server.receive_request());

        assert_eq!(debounce_ms.unwrap(), 250);
        assert!(matches!(request.unwrap(), Some(Request::GetDebounceMs)));
    }

    #[tokio::test]
    async fn client_receives_ignored_lints_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let document = Document::new_markdown_default_curated("A test document.");
        let lint = Lint::default();
        let mut config = Config::new();
        config.ignored_lints.ignore_lint(&lint, &document);
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(server_request_reader, server_response_writer, config);

        let (ignored_lints, request) =
            tokio::join!(client.get_ignored_lints(), server.receive_request());

        assert!(ignored_lints.unwrap().is_ignored(&lint, &document));
        assert!(matches!(request.unwrap(), Some(Request::GetIgnoredLints)));
    }

    #[tokio::test]
    async fn client_receives_integrations_from_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let mut config = Config::new();
        config.integrations = vec![crate::config::Integration {
            bundle_id: "com.example.Editor".to_string(),
            enabled: true,
        }];
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(server_request_reader, server_response_writer, config);

        let (integrations, request) =
            tokio::join!(client.get_integrations(), server.receive_request());

        assert_eq!(
            integrations.unwrap(),
            vec![crate::config::Integration {
                bundle_id: "com.example.Editor".to_string(),
                enabled: true,
            }]
        );
        assert!(matches!(request.unwrap(), Some(Request::GetIntegrations)));
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
                .await
                .ignored_lints
                .is_ignored(&lint, &document)
        );
    }

    #[tokio::test]
    async fn client_can_set_lint_config_on_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let config = Arc::new(Mutex::new(Config::new()));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );
        let mut expected = FlatConfig::new_curated();
        expected.set_rule_enabled("SpellCheck", false);

        let (ack, request) =
            tokio::join!(client.set_lint_config(&expected), server.receive_request());

        assert!(ack.is_ok());
        assert!(matches!(
            request.unwrap(),
            Some(Request::SetLintConfig { .. })
        ));
        assert_eq!(config.lock().await.lint_config, expected);
    }

    #[tokio::test]
    async fn client_can_disable_rule_on_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let config = Arc::new(Mutex::new(Config::new()));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );

        let disable = client.disable_rule("SpellCheck");
        let receive_requests = async {
            let first = server.receive_request().await;
            let second = server.receive_request().await;

            (first, second)
        };
        let (disable, (first_request, second_request)) = tokio::join!(disable, receive_requests);

        let updated = disable.unwrap();
        assert!(!updated.is_rule_enabled("SpellCheck"));
        assert!(matches!(
            first_request.unwrap(),
            Some(Request::GetLintConfig)
        ));
        assert!(matches!(
            second_request.unwrap(),
            Some(Request::SetLintConfig { .. })
        ));
        assert!(
            !config
                .lock()
                .await
                .lint_config
                .is_rule_enabled("SpellCheck")
        );
    }

    #[tokio::test]
    async fn client_can_add_word_to_server_dictionary() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let config = Arc::new(Mutex::new(Config::new()));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );

        let (ack, request) = tokio::join!(
            client.add_to_dictionary("blorple"),
            server.receive_request()
        );

        assert!(ack.is_ok());
        assert!(matches!(
            request.unwrap(),
            Some(Request::AddToDictionary { word }) if word == "blorple"
        ));
        assert!(
            config
                .lock()
                .await
                .mutable_dictionary
                .contains_word_str("blorple")
        );
    }

    #[tokio::test]
    async fn client_can_add_integration_on_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let config = Arc::new(Mutex::new(Config::new()));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );

        let (ack, request) = tokio::join!(
            client.add_integration("com.example.Editor"),
            server.receive_request()
        );

        assert!(ack.is_ok());
        assert!(matches!(
            request.unwrap(),
            Some(Request::AddIntegration { bundle_id }) if bundle_id == "com.example.Editor"
        ));
        assert!(
            config
                .lock()
                .await
                .is_integration_enabled("com.example.Editor")
        );
    }

    #[tokio::test]
    async fn client_can_remove_integration_on_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let mut config = Config::new();
        config.add_integration("com.example.Editor".to_string());
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );

        let (ack, request) = tokio::join!(
            client.remove_integration("com.example.Editor"),
            server.receive_request()
        );

        assert!(ack.is_ok());
        assert!(matches!(
            request.unwrap(),
            Some(Request::RemoveIntegration { bundle_id }) if bundle_id == "com.example.Editor"
        ));
        assert!(
            !config
                .lock()
                .await
                .integrations
                .iter()
                .any(|integration| integration.bundle_id == "com.example.Editor")
        );
    }

    #[tokio::test]
    async fn client_can_set_integration_enabled_on_server() {
        let (client_request_writer, server_request_reader) = duplex(16_384);
        let (server_response_writer, client_response_reader) = duplex(16_384);
        let mut config = Config::new();
        config.add_integration("com.example.Editor".to_string());
        let config = Arc::new(Mutex::new(config));
        let mut client = Client::new(client_response_reader, client_request_writer);
        let mut server = Server::new(
            server_request_reader,
            server_response_writer,
            config.clone(),
        );

        let (ack, request) = tokio::join!(
            client.set_integration_enabled("com.example.Editor", false),
            server.receive_request()
        );

        assert!(ack.is_ok());
        assert!(matches!(
            request.unwrap(),
            Some(Request::SetIntegrationEnabled { bundle_id, enabled })
                if bundle_id == "com.example.Editor" && !enabled
        ));
        assert!(
            !config
                .lock()
                .await
                .is_integration_enabled("com.example.Editor")
        );
    }

    #[tokio::test]
    async fn server_returns_none_on_eof() {
        let config = Arc::new(Mutex::new(Config::new()));
        let mut server = Server::new(empty(), sink(), config);

        assert!(server.receive_request().await.unwrap().is_none());
    }
}

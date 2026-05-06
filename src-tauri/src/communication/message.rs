use harper_core::{IgnoredLints, linting::FlatConfig};
use serde::{Deserialize, Serialize};

/// Canonical client-to-server protocol message sent by the highlighter process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    GetLintConfig,
    SetLintConfig { config: FlatConfig },
    IgnoreLint { ignored_lints: IgnoredLints },
    AddToDictionary { word: String },
}

/// Canonical server-to-client protocol message sent by the Tauri app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Response {
    GetLintConfig { config: FlatConfig },
    Ack,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetLintConfig).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetLintConfig));
    }

    #[test]
    fn ignore_lint_request_serializes_as_json() {
        let request = Request::IgnoreLint {
            ignored_lints: IgnoredLints::new(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::IgnoreLint { .. }));
    }

    #[test]
    fn set_lint_config_request_serializes_as_json() {
        let request = Request::SetLintConfig {
            config: FlatConfig::new_curated(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::SetLintConfig { .. }));
    }

    #[test]
    fn add_to_dictionary_request_serializes_as_json() {
        let request = Request::AddToDictionary {
            word: "blorple".to_string(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Request::AddToDictionary { word } if word == "blorple"
        ));
    }

    #[test]
    fn response_serializes_as_json() {
        let response = Response::GetLintConfig {
            config: FlatConfig::new_curated(),
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();

        assert_eq!(response, decoded);
    }

    #[test]
    fn ack_response_serializes_as_json() {
        let encoded = serde_json::to_string(&Response::Ack).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();

        assert_eq!(Response::Ack, decoded);
    }
}

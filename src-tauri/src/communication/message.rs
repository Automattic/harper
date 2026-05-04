use harper_core::{IgnoredLints, linting::FlatConfig};
use serde::{Deserialize, Serialize};

/// Canonical client-to-server protocol message sent by the highlighter process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    GetLintConfig,
    IgnoreLint { ignored_lints: IgnoredLints },
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

use harper_core::linting::FlatConfig;
use serde::{Deserialize, Serialize};

/// Canonical client-to-server protocol message sent by the highlighter process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Request {
    GetLintConfig,
}

/// Canonical server-to-client protocol message sent by the Tauri app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Response {
    GetLintConfig { config: FlatConfig },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetLintConfig).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();

        assert_eq!(Request::GetLintConfig, decoded);
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
}

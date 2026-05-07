use harper_core::{Dialect, IgnoredLints, linting::FlatConfig};
use serde::{Deserialize, Serialize};

/// Canonical client-to-server protocol message sent by the highlighter process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    GetLintConfig,
    GetDictionary,
    GetDialect,
    GetIgnoredLints,
    GetAllowedBundleIdentifiers,
    SetLintConfig { config: FlatConfig },
    IgnoreLint { ignored_lints: IgnoredLints },
    AddToDictionary { word: String },
    AddAllowedBundleIdentifier { bundle_identifier: String },
    RemoveAllowedBundleIdentifier { bundle_identifier: String },
}

/// Canonical server-to-client protocol message sent by the Tauri app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    GetLintConfig { config: FlatConfig },
    GetDictionary { words: Vec<String> },
    GetDialect { dialect: Dialect },
    GetIgnoredLints { ignored_lints: IgnoredLints },
    GetAllowedBundleIdentifiers { bundle_identifiers: Vec<String> },
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
    fn get_dictionary_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetDictionary).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetDictionary));
    }

    #[test]
    fn get_dialect_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetDialect).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetDialect));
    }

    #[test]
    fn get_ignored_lints_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetIgnoredLints).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetIgnoredLints));
    }

    #[test]
    fn get_allowed_bundle_identifiers_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetAllowedBundleIdentifiers).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetAllowedBundleIdentifiers));
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
    fn add_allowed_bundle_identifier_request_serializes_as_json() {
        let request = Request::AddAllowedBundleIdentifier {
            bundle_identifier: "com.example.Editor".to_string(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Request::AddAllowedBundleIdentifier { bundle_identifier }
                if bundle_identifier == "com.example.Editor"
        ));
    }

    #[test]
    fn remove_allowed_bundle_identifier_request_serializes_as_json() {
        let request = Request::RemoveAllowedBundleIdentifier {
            bundle_identifier: "com.example.Editor".to_string(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Request::RemoveAllowedBundleIdentifier { bundle_identifier }
                if bundle_identifier == "com.example.Editor"
        ));
    }

    #[test]
    fn response_serializes_as_json() {
        let response = Response::GetLintConfig {
            config: FlatConfig::new_curated(),
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Response::GetLintConfig { .. }));
    }

    #[test]
    fn dictionary_response_serializes_as_json() {
        let response = Response::GetDictionary {
            words: vec!["blorple".to_string()],
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Response::GetDictionary { words } if words == ["blorple"]));
    }

    #[test]
    fn dialect_response_serializes_as_json() {
        let response = Response::GetDialect {
            dialect: Dialect::British,
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Response::GetDialect {
                dialect: Dialect::British
            }
        ));
    }

    #[test]
    fn ignored_lints_response_serializes_as_json() {
        let response = Response::GetIgnoredLints {
            ignored_lints: IgnoredLints::new(),
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Response::GetIgnoredLints { .. }));
    }

    #[test]
    fn allowed_bundle_identifiers_response_serializes_as_json() {
        let response = Response::GetAllowedBundleIdentifiers {
            bundle_identifiers: vec!["com.example.Editor".to_string()],
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Response::GetAllowedBundleIdentifiers { bundle_identifiers }
                if bundle_identifiers == vec!["com.example.Editor".to_string()]
        ));
    }

    #[test]
    fn ack_response_serializes_as_json() {
        let encoded = serde_json::to_string(&Response::Ack).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Response::Ack));
    }
}

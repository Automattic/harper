use crate::config::Integration;
use harper_core::{Dialect, IgnoredLints, linting::FlatConfig};
use serde::{Deserialize, Serialize};

/// Canonical client-to-server protocol message sent by the highlighter process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    GetLintConfig,
    GetDictionary,
    GetDialect,
    GetDebounceMs,
    GetIgnoredLints,
    GetIntegrations,
    SetLintConfig { config: FlatConfig },
    IgnoreLint { ignored_lints: IgnoredLints },
    AddToDictionary { word: String },
    AddIntegration { bundle_id: String },
    RemoveIntegration { bundle_id: String },
    SetIntegrationEnabled { bundle_id: String, enabled: bool },
}

/// Canonical server-to-client protocol message sent by the Tauri app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    GetLintConfig { config: FlatConfig },
    GetDictionary { words: Vec<String> },
    GetDialect { dialect: Dialect },
    GetDebounceMs { debounce_ms: u64 },
    GetIgnoredLints { ignored_lints: IgnoredLints },
    GetIntegrations { integrations: Vec<Integration> },
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
    fn get_debounce_ms_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetDebounceMs).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetDebounceMs));
    }

    #[test]
    fn get_ignored_lints_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetIgnoredLints).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetIgnoredLints));
    }

    #[test]
    fn get_integrations_request_serializes_as_json() {
        let encoded = serde_json::to_string(&Request::GetIntegrations).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Request::GetIntegrations));
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
    fn add_integration_request_serializes_as_json() {
        let request = Request::AddIntegration {
            bundle_id: "com.example.Editor".to_string(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Request::AddIntegration { bundle_id } if bundle_id == "com.example.Editor"
        ));
    }

    #[test]
    fn remove_integration_request_serializes_as_json() {
        let request = Request::RemoveIntegration {
            bundle_id: "com.example.Editor".to_string(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Request::RemoveIntegration { bundle_id } if bundle_id == "com.example.Editor"
        ));
    }

    #[test]
    fn set_integration_enabled_request_serializes_as_json() {
        let request = Request::SetIntegrationEnabled {
            bundle_id: "com.example.Editor".to_string(),
            enabled: false,
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Request::SetIntegrationEnabled { bundle_id, enabled }
                if bundle_id == "com.example.Editor" && !enabled
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
    fn debounce_ms_response_serializes_as_json() {
        let response = Response::GetDebounceMs { debounce_ms: 250 };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Response::GetDebounceMs { debounce_ms } if debounce_ms == 250));
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
    fn integrations_response_serializes_as_json() {
        let response = Response::GetIntegrations {
            integrations: vec![Integration {
                bundle_id: "com.example.Editor".to_string(),
                enabled: true,
            }],
        };
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(
            decoded,
            Response::GetIntegrations { integrations }
                if integrations == vec![Integration {
                    bundle_id: "com.example.Editor".to_string(),
                    enabled: true,
                }]
        ));
    }

    #[test]
    fn ack_response_serializes_as_json() {
        let encoded = serde_json::to_string(&Response::Ack).unwrap();
        let decoded: Response = serde_json::from_str(&encoded).unwrap();

        assert!(matches!(decoded, Response::Ack));
    }
}

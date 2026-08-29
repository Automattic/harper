use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integration {
    pub bundle_id: String,
    pub enabled: bool,
}

impl Integration {
    pub fn curated_integrations() -> Vec<Self> {
        #[cfg(target_os = "macos")]
        let integrations = [
            "com.apple.TextEdit",
            "com.apple.mail",
            "com.apple.MobileSMS",
            "com.apple.Notes",
            "com.tinyspeck.slackmacgap",
            "com.hnc.Discord",
        ];

        #[cfg(target_os = "windows")]
        let integrations: [&str; 0] = [];

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let integrations: [&str; 0] = [];

        integrations
            .into_iter()
            .map(|bundle_id| Integration {
                bundle_id: bundle_id.to_string(),
                enabled: true,
            })
            .collect()
    }
    pub fn is_integration_enabled_in(integrations: &[Self], bundle_id: &str) -> bool {
        integrations
            .iter()
            .any(|integration| integration.bundle_id == bundle_id && integration.enabled)
    }
}

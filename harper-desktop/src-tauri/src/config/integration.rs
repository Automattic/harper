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
            "ru.keepcoder.Telegram",
            "com.bloombuilt.dayone-mac",
        ];

        #[cfg(target_os = "windows")]
        let integrations: [&str; 0] = [];

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let integrations: [&str; 0] = [];

        integrations
            .into_iter()
            .map(|bundle_id| Integration {
                bundle_id: bundle_id.to_string(),
                // Onboarding teaches users to enable integrations by enabling TextEdit,
                // so it starts disabled while the other curated integrations start enabled.
                enabled: bundle_id != "com.apple.TextEdit",
            })
            .collect()
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::Integration;

    #[test]
    fn telegram_is_enabled_by_default() {
        assert!(
            Integration::curated_integrations()
                .iter()
                .any(|integration| {
                    integration.bundle_id == "ru.keepcoder.Telegram" && integration.enabled
                })
        );
    }
}

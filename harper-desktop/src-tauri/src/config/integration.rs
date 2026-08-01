use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integration {
    pub bundle_id: String,
    pub enabled: bool,
}

impl Integration {
    pub fn curated_integrations() -> Vec<Self> {
        #[cfg(target_os = "macos")]
        let ids: &[&str] = &[
            "com.apple.TextEdit",
            "com.apple.mail",
            "com.apple.MobileSMS",
            "com.apple.Notes",
            "com.tinyspeck.slackmacgap",
            "com.hnc.Discord",
        ];

        #[cfg(windows)]
        let ids: &[&str] = &[
            "notepad.exe",
            "notepad++.exe",
            "slack.exe",
            "discord.exe",
            "applicationframehost.exe",
        ];

        #[cfg(not(any(target_os = "macos", windows)))]
        let ids: &[&str] = &[];

        ids.iter()
            .map(|&id| Integration {
                bundle_id: id.to_string(),
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

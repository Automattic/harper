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
            "com.bloombuilt.dayone-mac",
        ];

        #[cfg(target_os = "windows")]
        let integrations = ["notepad.exe"];

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
        integrations.iter().any(|integration| {
            if !integration.enabled {
                return false;
            }
            if integration.bundle_id == bundle_id {
                return true;
            }
            #[cfg(target_os = "windows")]
            {
                if integration.bundle_id.eq_ignore_ascii_case(bundle_id) {
                    return true;
                }
                let integration_name = std::path::Path::new(&integration.bundle_id)
                    .file_name()
                    .and_then(|f| f.to_str());
                let target_name = std::path::Path::new(bundle_id)
                    .file_name()
                    .and_then(|f| f.to_str());
                if let (Some(a), Some(b)) = (integration_name, target_name)
                    && a.eq_ignore_ascii_case(b)
                {
                    return true;
                }
            }
            false
        })
    }
}

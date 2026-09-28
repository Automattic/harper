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
        let integrations = ["notepad.exe", "winword.exe"];

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

    pub fn matches_bundle_id(configured: &str, target: &str) -> bool {
        let configured = configured.trim();
        let target = target.trim();
        if configured.is_empty() || target.is_empty() {
            return false;
        }
        if configured == target {
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            if configured.eq_ignore_ascii_case(target) {
                return true;
            }
            let int_path = std::path::Path::new(configured);
            let target_path = std::path::Path::new(target);

            let int_file = int_path.file_name().and_then(|f| f.to_str());
            let target_file = target_path.file_name().and_then(|f| f.to_str());
            if let (Some(a), Some(b)) = (int_file, target_file)
                && a.eq_ignore_ascii_case(b)
            {
                return true;
            }

            let int_stem = int_path.file_stem().and_then(|f| f.to_str());
            let target_stem = target_path.file_stem().and_then(|f| f.to_str());
            if let (Some(a), Some(b)) = (int_stem, target_stem)
                && a.eq_ignore_ascii_case(b)
            {
                return true;
            }
        }
        false
    }

    pub fn find_integration<'a>(integrations: &'a [Self], bundle_id: &str) -> Option<&'a Self> {
        integrations
            .iter()
            .find(|item| Self::matches_bundle_id(&item.bundle_id, bundle_id))
    }

    pub fn find_integration_mut<'a>(
        integrations: &'a mut [Self],
        bundle_id: &str,
    ) -> Option<&'a mut Self> {
        integrations
            .iter_mut()
            .find(|item| Self::matches_bundle_id(&item.bundle_id, bundle_id))
    }

    pub fn is_integration_enabled_in(integrations: &[Self], bundle_id: &str) -> bool {
        Self::find_integration(integrations, bundle_id).is_some_and(|item| item.enabled)
    }
}

use base64::{Engine as _, engine::general_purpose::STANDARD};
use harper_core::weirpack::Weirpack;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// An installed pack, shared through settings and IPC. Renaming changes only the display name.
/// Original archive bytes are retained so dictionaries and rules can be rebuilt after removal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledWeirpack {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub version: String,
    pub size: usize,
    pub rule_count: usize,
    pub added_at: u64,
    pub enabled: bool,
    pub bytes_base64: String,
}

impl InstalledWeirpack {
    /// Validate the archive, rule tests, and dictionary before creating an enabled installation.
    /// Invalid packs never enter the canonical configuration.
    pub fn install(filename: String, bytes: Vec<u8>) -> Result<Self, String> {
        let pack = Weirpack::from_bytes(&bytes).map_err(|error| error.to_string())?;
        let failures = pack.run_tests().map_err(|error| error.to_string())?;
        if !failures.is_empty() {
            return Err(format!(
                "Weirpack tests failed for {} rule(s).",
                failures.len()
            ));
        }
        pack.load_dictionary().map_err(|error| error.to_string())?;
        pack.to_lint_group().map_err(|error| error.to_string())?;
        Ok(Self {
            id: format!("{:032x}", fastrand::u128(..)),
            name: pack
                .manifest
                .get_field("name")
                .and_then(serde_json::Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or(filename.strip_suffix(".weirpack").unwrap_or(&filename))
                .to_owned(),
            version: pack
                .manifest
                .version()
                .map_err(|error| error.to_string())?
                .to_owned(),
            filename,
            size: bytes.len(),
            rule_count: pack.rules.len(),
            added_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis() as u64,
            enabled: true,
            bytes_base64: STANDARD.encode(bytes),
        })
    }

    pub fn unpack(&self) -> Result<Weirpack, String> {
        let bytes = STANDARD
            .decode(&self.bytes_base64)
            .map_err(|error| error.to_string())?;
        Weirpack::from_bytes(&bytes).map_err(|error| error.to_string())
    }
}

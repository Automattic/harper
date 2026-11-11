use base64::{Engine as _, engine::general_purpose::STANDARD};
use harper_core::weirpack::Weirpack;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
pub(crate) fn test_weirpack() -> Weirpack {
    let mut manifest = harper_core::weirpack::WeirpackManifest::new();
    manifest.set_author("Desktop tests");
    manifest.set_version("1.0.0");
    manifest.set_description("Test pack");
    manifest.set_license("MIT");
    let mut pack = Weirpack::new(manifest);
    pack.add_rule(
        "DesktopTestRule",
        r#"
expr main banana
let message "Use apple."
let description "Test replacement."
let becomes "apple"
test "banana" "apple"
"#,
    );
    pack
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_preserves_archive_and_metadata() {
        let bytes = test_weirpack().to_bytes().unwrap();
        let installed = InstalledWeirpack::install("test.weirpack".into(), bytes.clone()).unwrap();
        assert_eq!(installed.name, "test");
        assert_eq!(installed.version, "1.0.0");
        assert_eq!(installed.rule_count, 1);
        assert_eq!(installed.size, bytes.len());
        assert!(installed.enabled);
        assert_eq!(STANDARD.decode(&installed.bytes_base64).unwrap(), bytes);
        assert_eq!(installed.unpack().unwrap().rules, test_weirpack().rules);
    }

    #[test]
    fn install_uses_manifest_name() {
        let mut pack = test_weirpack();
        pack.manifest
            .set_field("name", serde_json::json!("Style guide"));
        let installed =
            InstalledWeirpack::install("test.weirpack".into(), pack.to_bytes().unwrap()).unwrap();
        assert_eq!(installed.name, "Style guide");
    }

    #[test]
    fn rejects_invalid_archive() {
        assert!(InstalledWeirpack::install("bad.weirpack".into(), b"not zip".to_vec()).is_err());
    }

    #[test]
    fn rejects_invalid_rule() {
        let mut pack = test_weirpack();
        pack.add_rule("Broken", "not valid Weir");
        assert!(
            InstalledWeirpack::install("bad.weirpack".into(), pack.to_bytes().unwrap()).is_err()
        );
    }

    #[test]
    fn rejects_failing_tests() {
        let mut pack = test_weirpack();
        pack.rules
            .get_mut("DesktopTestRule")
            .unwrap()
            .push_str("\ntest \"banana\" \"pear\"");
        let error = InstalledWeirpack::install("bad.weirpack".into(), pack.to_bytes().unwrap())
            .unwrap_err();
        assert!(error.contains("tests failed"));
    }

    #[test]
    fn rejects_invalid_dictionary() {
        let mut pack = test_weirpack();
        pack.dictionary = Some("blorple".into());
        pack.annotations = Some("not JSON".into());
        assert!(
            InstalledWeirpack::install("bad.weirpack".into(), pack.to_bytes().unwrap()).is_err()
        );
    }

    #[test]
    fn installations_have_distinct_ids() {
        let bytes = test_weirpack().to_bytes().unwrap();
        let first = InstalledWeirpack::install("test.weirpack".into(), bytes.clone()).unwrap();
        let second = InstalledWeirpack::install("test.weirpack".into(), bytes).unwrap();
        assert_ne!(first.id, second.id);
    }
}

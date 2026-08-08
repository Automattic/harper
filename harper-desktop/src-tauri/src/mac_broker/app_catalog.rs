use std::{collections::BTreeSet, path::Path, process::Command};

use crate::os_broker::AppSearchResult;

const APPLICATION_BUNDLE_CONTENT_TYPE: &str = "com.apple.application-bundle";

pub fn app_search_result_from_bundle_id(bundle_id: &str) -> AppSearchResult {
    AppSearchResult {
        name: system_integration_display_name(bundle_id),
        bundle_id: bundle_id.to_string(),
    }
}

pub fn system_integration_display_name(bundle_id: &str) -> String {
    application_path_for_bundle_id(bundle_id)
        .and_then(|path| display_name_from_app_path(&path))
        .unwrap_or_else(|| bundle_id.to_owned())
}

fn display_name_from_app_path(path: &str) -> Option<String> {
    let file_name = Path::new(path).file_name()?.to_str()?;
    let display_name = file_name.strip_suffix(".app").unwrap_or(file_name).trim();

    if display_name.is_empty() {
        None
    } else {
        Some(display_name.to_string())
    }
}

pub fn installed_application_bundle_ids() -> Result<Vec<String>, String> {
    let output = Command::new("mdfind")
        .arg(format!(
            "kMDItemContentType == \"{APPLICATION_BUNDLE_CONTENT_TYPE}\""
        ))
        .output()
        .map_err(|error| format!("Failed to list installed applications: {error}"))?;

    if !output.status.success() {
        return Err("Failed to list installed applications with Spotlight.".to_string());
    }

    let bundle_ids = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| bundle_id_from_app_path(line.trim()))
        .collect::<Vec<_>>();

    Ok(deduplicate_and_sort_bundle_ids(bundle_ids))
}

/// Basic validation for macOS bundle identifiers (reverse-DNS-like).
fn is_valid_bundle_id(bundle_id: &str) -> bool {
    // must contain at least one dot, and only allow alnum / '-' / '_' / '.'
    if !bundle_id.contains('.') {
        return false;
    }
    bundle_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// Sanitize an icon filename read from an Info.plist. Accept only simple filenames (no slashes),
/// disallow absolute paths and parent-directory components.
fn sanitize_icon_file(name: &str) -> Option<String> {
    let s = name.trim();
    if s.is_empty() {
        return None;
    }
    if s.contains('/') || s.contains('\\') || s.contains("..") {
        return None;
    }
    Some(s.to_string())
}

/// Resolves a bundle identifier to an installed `.app` path using Spotlight metadata.
pub fn application_path_for_bundle_id(bundle_id: &str) -> Option<String> {
    let bundle_id = bundle_id.trim();

    if bundle_id.is_empty() {
        return None;
    }
    
    // Validate bundle ID – avoid passing arbitrary values to mdfind.
    if !is_valid_bundle_id(bundle_id) {
        return None;
    }
    
    let predicate_bundle_id = escape_spotlight_string(bundle_id);
    let output = Command::new("mdfind")
        .arg(format!(
            "kMDItemCFBundleIdentifier == \"{predicate_bundle_id}\""
        ))
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && line.ends_with(".app"))
        .map(ToString::to_string)
}

fn bundle_id_from_app_path(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    let output = Command::new("mdls")
        .arg("-raw")
        .arg("-name")
        .arg("kMDItemCFBundleIdentifier")
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if bundle_id.is_empty() || bundle_id == "(null)" {
        None
    } else {
        Some(bundle_id)
    }
}

fn escape_spotlight_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn deduplicate_and_sort_bundle_ids(bundle_ids: Vec<String>) -> Vec<String> {
    bundle_ids
        .into_iter()
        .map(|bundle_id| bundle_id.trim().to_string())
        .filter(|bundle_id| !bundle_id.is_empty() && bundle_id != "(null)")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_and_sorts_bundle_ids() {
        assert_eq!(
            deduplicate_and_sort_bundle_ids(vec![
                "com.example.Beta".to_string(),
                " com.example.Alpha ".to_string(),
                "com.example.Beta".to_string(),
                "".to_string(),
                "(null)".to_string(),
            ]),
            vec!["com.example.Alpha", "com.example.Beta"]
        );
    }

    #[test]
    fn escapes_spotlight_strings() {
        assert_eq!(
            escape_spotlight_string(r#"com.example.\"quoted\""#),
            r#"com.example.\\\"quoted\\\""#
        );
    }

    #[test]
    fn empty_bundle_id_has_no_application_path() {
        assert_eq!(application_path_for_bundle_id(""), None);
        assert_eq!(application_path_for_bundle_id("   "), None);
    }
}

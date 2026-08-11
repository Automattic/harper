use plist::{Dictionary, Value};
use std::{collections::BTreeSet, path::Path, process::Command};

use crate::os_broker::AppSearchResult;

const APPLICATION_BUNDLE_CONTENT_TYPE: &str = "com.apple.application-bundle";

pub fn system_integration_display_name(bundle_id: &str) -> String {
    application_path_for_bundle_id(bundle_id)
        .and_then(|path| {
            let path = Path::new(&path);
            app_search_result_from_app_path(path)
                .map(|application| application.name)
                .ok()
                .or_else(|| display_name_from_app_path(path))
        })
        .unwrap_or_else(|| bundle_id.to_owned())
}

fn display_name_from_app_path(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let display_name = file_name.strip_suffix(".app").unwrap_or(file_name).trim();

    if display_name.is_empty() {
        None
    } else {
        Some(display_name.to_string())
    }
}

pub fn installed_applications() -> Result<Vec<AppSearchResult>, String> {
    let output = Command::new("mdfind")
        .arg(format!(
            "kMDItemContentType == \"{APPLICATION_BUNDLE_CONTENT_TYPE}\""
        ))
        .output()
        .map_err(|error| format!("Failed to list installed applications: {error}"))?;

    if !output.status.success() {
        return Err("Failed to list installed applications with Spotlight.".to_string());
    }

    let applications = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| app_search_result_from_app_path(Path::new(line.trim())).ok())
        .collect::<Vec<_>>();

    Ok(deduplicate_and_sort_applications(applications))
}

pub fn app_search_result_from_path(path: &str) -> Result<AppSearchResult, String> {
    let path = path.trim();

    if path.is_empty() {
        return Err("Application path cannot be empty.".to_string());
    }

    let app_path = Path::new(path);
    if !app_path.is_dir() {
        return Err("The selected application is not a directory.".to_string());
    }

    if app_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("app")
    {
        return Err("The selected directory must be an .app bundle.".to_string());
    }

    app_search_result_from_app_path(app_path)
}

/// Resolves a bundle identifier to an installed `.app` path using Spotlight metadata.
pub fn application_path_for_bundle_id(bundle_id: &str) -> Option<String> {
    let bundle_id = bundle_id.trim();

    if bundle_id.is_empty() {
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

fn app_search_result_from_app_path(path: &Path) -> Result<AppSearchResult, String> {
    let info = Value::from_file(path.join("Contents/Info.plist"))
        .map_err(|error| format!("Unable to read application Info.plist: {error}"))?;
    let info = info
        .as_dictionary()
        .ok_or_else(|| "Application Info.plist is not a dictionary.".to_string())?;
    let bundle_id = plist_string(info, "CFBundleIdentifier")
        .ok_or_else(|| "Application Info.plist does not contain a bundle ID.".to_string())?
        .to_string();
    let name = plist_string(info, "CFBundleDisplayName")
        .or_else(|| plist_string(info, "CFBundleName"))
        .map(ToString::to_string)
        .or_else(|| display_name_from_app_path(path))
        .ok_or_else(|| "The selected application does not have a display name.".to_string())?;

    Ok(AppSearchResult { name, bundle_id })
}

fn plist_string<'a>(dictionary: &'a Dictionary, key: &str) -> Option<&'a str> {
    dictionary
        .get(key)?
        .as_string()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn escape_spotlight_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn deduplicate_and_sort_applications(
    mut applications: Vec<AppSearchResult>,
) -> Vec<AppSearchResult> {
    applications.sort_by_cached_key(|application| {
        (
            application.name.to_lowercase(),
            application.name.clone(),
            application.bundle_id.to_lowercase(),
            application.bundle_id.clone(),
        )
    });

    let mut seen_bundle_ids = BTreeSet::new();
    applications.retain(|application| seen_bundle_ids.insert(application.bundle_id.clone()));
    applications
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn write_app_fixture(
        root: &Path,
        filename: &str,
        bundle_id: &str,
        display_name: Option<&str>,
        bundle_name: Option<&str>,
        binary: bool,
    ) -> PathBuf {
        let app_path = root.join(filename);
        let contents_path = app_path.join("Contents");
        fs::create_dir_all(&contents_path).unwrap();

        let mut info = Dictionary::new();
        info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String(bundle_id.to_string()),
        );
        if let Some(display_name) = display_name {
            info.insert(
                "CFBundleDisplayName".to_string(),
                Value::String(display_name.to_string()),
            );
        }
        if let Some(bundle_name) = bundle_name {
            info.insert(
                "CFBundleName".to_string(),
                Value::String(bundle_name.to_string()),
            );
        }

        let info = Value::Dictionary(info);
        let info_path = contents_path.join("Info.plist");
        if binary {
            info.to_file_binary(info_path).unwrap();
        } else {
            info.to_file_xml(info_path).unwrap();
        }

        app_path
    }

    #[test]
    fn deduplicates_and_sorts_applications() {
        assert_eq!(
            deduplicate_and_sort_applications(vec![
                AppSearchResult {
                    name: "TextEdit".to_string(),
                    bundle_id: "com.apple.TextEdit".to_string(),
                },
                AppSearchResult {
                    name: "Safari".to_string(),
                    bundle_id: "com.apple.Safari".to_string(),
                },
                AppSearchResult {
                    name: "TextEdit Duplicate".to_string(),
                    bundle_id: "com.apple.TextEdit".to_string(),
                },
            ]),
            vec![
                AppSearchResult {
                    name: "Safari".to_string(),
                    bundle_id: "com.apple.Safari".to_string(),
                },
                AppSearchResult {
                    name: "TextEdit".to_string(),
                    bundle_id: "com.apple.TextEdit".to_string(),
                },
            ]
        );
    }

    #[test]
    fn resolves_synthetic_xml_and_binary_app_bundles_without_spotlight() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("harper-app-catalog-{}-{nonce}", std::process::id()));
        let display_app = write_app_fixture(
            &root,
            "Display.app",
            "com.example.Display",
            Some("Preferred Display Name"),
            Some("Ignored Bundle Name"),
            true,
        );
        let named_app = write_app_fixture(
            &root,
            "Named.app",
            "com.example.Named",
            None,
            Some("Bundle Name"),
            false,
        );
        let filename_app = write_app_fixture(
            &root,
            "Filename.app",
            "com.example.Filename",
            None,
            None,
            false,
        );

        let display_result = app_search_result_from_path(display_app.to_str().unwrap()).unwrap();
        let named_result = app_search_result_from_path(named_app.to_str().unwrap()).unwrap();
        let filename_result = app_search_result_from_path(filename_app.to_str().unwrap()).unwrap();
        fs::remove_dir_all(root).unwrap();

        assert_eq!(display_result.name, "Preferred Display Name");
        assert_eq!(display_result.bundle_id, "com.example.Display");
        assert_eq!(named_result.name, "Bundle Name");
        assert_eq!(named_result.bundle_id, "com.example.Named");
        assert_eq!(filename_result.name, "Filename");
        assert_eq!(filename_result.bundle_id, "com.example.Filename");
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

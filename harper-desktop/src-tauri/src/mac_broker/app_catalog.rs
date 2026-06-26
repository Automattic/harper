use std::{collections::BTreeSet, path::Path, process::Command};

use crate::os_broker::{AppSearchResult, OsBroker};

use super::MacBroker;

const APPLICATION_BUNDLE_CONTENT_TYPE: &str = "com.apple.application-bundle";

pub(super) fn app_search_result_from_bundle_id(
    broker: &MacBroker,
    bundle_id: &str,
) -> AppSearchResult {
    AppSearchResult {
        name: broker.integration_display_name(bundle_id),
        bundle_id: bundle_id.to_string(),
    }
}

pub(super) fn app_search_result_from_app_path(path: &str) -> Option<AppSearchResult> {
    let display_name = display_name_from_app_path(path)?;

    if display_name.contains('.') {
        return None;
    }

    let bundle_id = bundle_id_from_app_path(path)?;

    if bundle_id.contains('/') || bundle_id.is_empty() {
        return None;
    }

    Some(AppSearchResult {
        name: display_name,
        bundle_id,
    })
}

pub(super) fn discover_app_search_results() -> Result<Vec<AppSearchResult>, String> {
    let output = Command::new("mdfind")
        .arg(format!(
            "kMDItemContentType == '{APPLICATION_BUNDLE_CONTENT_TYPE}'"
        ))
        .output()
        .map_err(|error| format!("Failed to execute mdfind: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "mdfind failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let mut results = Vec::new();
    let mut seen_bundle_ids = BTreeSet::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();

        if !line.ends_with(".app") {
            continue;
        }

        let Some(result) = app_search_result_from_app_path(line) else {
            continue;
        };

        if seen_bundle_ids.insert(result.bundle_id.clone()) {
            results.push(result);
        }
    }

    Ok(results)
}

pub(super) fn system_integration_display_name(bundle_id: &str) -> Option<String> {
    application_path_for_bundle_id(bundle_id).and_then(|path| display_name_from_app_path(&path))
}

pub(super) fn installed_application_bundle_ids() -> Result<Vec<String>, String> {
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

pub(super) fn application_path_for_bundle_id(bundle_id: &str) -> Option<String> {
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

pub(super) fn escape_spotlight_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn deduplicate_and_sort_bundle_ids(bundle_ids: Vec<String>) -> Vec<String> {
    bundle_ids
        .into_iter()
        .map(|bundle_id| bundle_id.trim().to_string())
        .filter(|bundle_id| !bundle_id.is_empty() && bundle_id != "(null)")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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

pub(super) fn launch_app_bundle(bundle_id: &str) -> Result<(), String> {
    let bundle_id = bundle_id.trim();

    if bundle_id.is_empty() {
        return Err("Bundle ID cannot be empty.".to_string());
    }

    Command::new("open")
        .arg("-b")
        .arg(bundle_id)
        .spawn()
        .map_err(|error| format!("Failed to launch {bundle_id}: {error}"))?;

    Ok(())
}

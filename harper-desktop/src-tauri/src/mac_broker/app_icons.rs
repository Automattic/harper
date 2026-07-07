use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use super::app_catalog::application_path_for_bundle_id;

/// Converts an installed app's icon resource to PNG bytes using the macOS `sips` tool.
pub fn application_icon_png(bundle_id: &str) -> Result<Vec<u8>, String> {
    let bundle_id = bundle_id.trim();

    if bundle_id.is_empty() {
        return Err("Bundle ID cannot be empty.".to_string());
    }

    let app_path = application_path_for_bundle_id(bundle_id)
        .ok_or_else(|| format!("No application found for bundle ID {bundle_id}."))?;
    let icon_path = icon_path_for_app(&app_path)
        .ok_or_else(|| format!("No application icon found for bundle ID {bundle_id}."))?;
    let output_path = std::env::temp_dir().join(format!(
        "harper-{bundle_id}-icon-{}.png",
        std::process::id()
    ));

    let output = Command::new("sips")
        .arg("-s")
        .arg("format")
        .arg("png")
        .arg(&icon_path)
        .arg("--out")
        .arg(&output_path)
        .output()
        .map_err(|error| format!("Failed to convert icon for {bundle_id}: {error}"))?;

    if !output.status.success() {
        return Err(format!("Failed to convert icon for {bundle_id} to PNG."));
    }

    let bytes = fs::read(&output_path)
        .map_err(|error| format!("Failed to read converted icon for {bundle_id}: {error}"))?;
    let _ = fs::remove_file(output_path);

    Ok(bytes)
}

fn icon_path_for_app(app_path: &str) -> Option<PathBuf> {
    let resources_path = Path::new(app_path).join("Contents/Resources");
    let icon_file = app_icon_file(app_path)?;
    let icon_path = resources_path.join(&icon_file);

    if icon_path.exists() {
        return Some(icon_path);
    }

    let icon_path = resources_path.join(format!("{icon_file}.icns"));

    if icon_path.exists() {
        Some(icon_path)
    } else {
        None
    }
}

fn app_icon_file(app_path: &str) -> Option<String> {
    let info_plist_path = Path::new(app_path).join("Contents/Info.plist");
    let output = Command::new("/usr/libexec/PlistBuddy")
        .arg("-c")
        .arg("Print :CFBundleIconFile")
        .arg(info_plist_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let icon_file = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if icon_file.is_empty() {
        None
    } else {
        Some(icon_file)
    }
}

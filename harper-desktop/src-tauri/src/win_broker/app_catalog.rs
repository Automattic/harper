use std::{process::Command, sync::OnceLock};

use crate::os_broker::AppSearchResult;

const START_MENU_SCRIPT: &str = r#"
$shell = New-Object -ComObject WScript.Shell
$startMenuPaths = @(
    "$env:ProgramData\Microsoft\Windows\Start Menu\Programs",
    "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
)
$apps = Get-ChildItem -Path $startMenuPaths -Recurse -Filter *.lnk -ErrorAction SilentlyContinue | ForEach-Object {
    $target = $shell.CreateShortcut($_.FullName).TargetPath
    if ($target -match '\.exe$') {
        [PSCustomObject]@{
            Name = $_.BaseName
            ExeName = [System.IO.Path]::GetFileName($target).ToLower()
            Path = $target
        }
    }
} | Group-Object ExeName | ForEach-Object {
    $_.Group[0]
}
$apps | ConvertTo-Json -Depth 2
"#;

#[derive(serde::Deserialize, Debug, Clone)]
struct PsAppEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ExeName")]
    pub exe_name: String,
    #[serde(rename = "Path")]
    pub path: String,
}

static CATALOG_CACHE: OnceLock<Vec<PsAppEntry>> = OnceLock::new();

fn get_app_catalog() -> &'static [PsAppEntry] {
    CATALOG_CACHE.get_or_init(|| {
        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(START_MENU_SCRIPT)
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let apps: Vec<PsAppEntry> = serde_json::from_str(&stdout).unwrap_or_default();
        apps
    })
}

pub fn search_apps(query: &str) -> Result<Vec<AppSearchResult>, String> {
    let query_lower = query.to_lowercase();
    let apps = get_app_catalog();

    let mut results = Vec::new();
    for app in apps {
        if query_lower.is_empty() || app.name.to_lowercase().contains(&query_lower) {
            results.push(AppSearchResult {
                name: app.name.clone(),
                bundle_id: app.exe_name.clone(),
            });
        }
    }

    // Sort alphabetically by name
    results.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(results)
}

pub fn system_integration_display_name(bundle_id: &str) -> String {
    let bundle_id_lower = bundle_id.to_lowercase();
    get_app_catalog()
        .iter()
        .find(|app| app.exe_name == bundle_id_lower)
        .map(|app| app.name.clone())
        .unwrap_or_else(|| bundle_id.to_string())
}

pub fn application_path_for_bundle_id(bundle_id: &str) -> Option<String> {
    let bundle_id_lower = bundle_id.to_lowercase();
    get_app_catalog()
        .iter()
        .find(|app| app.exe_name == bundle_id_lower)
        .map(|app| app.path.clone())
}

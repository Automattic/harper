use std::process::Command;

pub fn application_icon_png(app_path: &str) -> Result<Vec<u8>, String> {
    // Path is passed via environment variable to avoid PowerShell string injection.
    let script = r#"
Add-Type -AssemblyName System.Drawing
$target = $env:HARPER_APP_PATH
try {
    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($target)
    if ($null -ne $icon) {
        $bmp = $icon.ToBitmap()
        $ms = New-Object System.IO.MemoryStream
        $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Output ([Convert]::ToBase64String($ms.ToArray()))
    }
} catch {
    exit 1
}
"#;

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .env("HARPER_APP_PATH", app_path)
        .output()
        .map_err(|e| format!("Failed to extract icon using PowerShell: {e}"))?;

    if !output.status.success() {
        return Err(format!("Icon extraction failed for path: {app_path}"));
    }

    let b64 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if b64.is_empty() {
        return Err(format!("Empty icon extracted for path: {app_path}"));
    }

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD
        .decode(b64)
        .map_err(|e| format!("Failed to decode Base64 icon: {e}"))
}

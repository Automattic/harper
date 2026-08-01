use std::process::Command;

pub fn application_icon_png(app_path: &str) -> Result<Vec<u8>, String> {
    let script = format!(
        r#"
Add-Type -AssemblyName System.Drawing
$target = "{app_path}"
try {{
    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($target)
    if ($null -ne $icon) {{
        $bmp = $icon.ToBitmap()
        $ms = New-Object System.IO.MemoryStream
        $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Output ([Convert]::ToBase64String($ms.ToArray()))
    }}
}} catch {{
    exit 1
}}
"#
    );

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
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

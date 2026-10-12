$ErrorActionPreference = 'Stop'

function Invoke-Checked([scriptblock] $Command) {
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "Command failed (exit $LASTEXITCODE): $Command" }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Set-Location $repoRoot
$config = Get-Content 'harper-desktop/src-tauri/tauri.conf.json' -Raw | ConvertFrom-Json

Write-Host '--- :rust: Prepare Rust and build tools'
$cargoBin = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME 'bin' } else { Join-Path $env:USERPROFILE '.cargo/bin' }
$env:PATH = "$cargoBin;$env:PATH"
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    # rustup selects installer mode from its executable name.
    $rustupInstaller = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile $rustupInstaller
    Invoke-Checked { & $rustupInstaller -y --no-modify-path --default-toolchain stable --profile minimal }
}
Invoke-Checked { rustup update stable --no-self-update }
Invoke-Checked { rustup target add wasm32-unknown-unknown x86_64-pc-windows-msvc }

if (-not (Get-Command cargo-binstall -ErrorAction SilentlyContinue)) {
    $binstallInstaller = Join-Path $env:TEMP 'harper-install-binstall.ps1'
    Invoke-WebRequest 'https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.ps1' -OutFile $binstallInstaller -UseBasicParsing
    Invoke-Checked { & $binstallInstaller }
}
Invoke-Checked { cargo binstall --no-confirm --force just wasm-pack }
Invoke-Checked { npm install --global pnpm@10.10.0 }
$pnpmPrefix = Invoke-Checked { npm prefix --global }
$env:PATH = "$pnpmPrefix;$env:PATH"

Write-Host '--- :lock: Set up Azure Trusted Signing'
$setupScript = (Get-Command setup_azure_trusted_signing.ps1 -ErrorAction Stop).Source
Invoke-Checked { & $setupScript }

# Absolute paths and separate arguments handle NSIS's working directory and spaces.
$signingConfig = Join-Path $env:TEMP "harper-windows-signing-$([guid]::NewGuid()).json"
$overlay = @{
    bundle = @{
        createUpdaterArtifacts = $false
        windows = @{
            signCommand = @{
                cmd = (Get-Command node -ErrorAction Stop).Source
                args = @((Join-Path $PSScriptRoot 'sign-windows.cjs'), '%1')
            }
        }
    }
} | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($signingConfig, $overlay, [System.Text.UTF8Encoding]::new($false))

try {
    Write-Host '--- :tauri: Build and sign Harper for Windows'
    Invoke-Checked { just build-desktop-windows-signed $signingConfig }

    $bundleDir = 'harper-desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis'
    $installers = @(Get-ChildItem $bundleDir -Filter '*-setup.exe' -File)
    $expectedName = "Harper_$($config.version)_x64-setup.exe"
    if ($installers.Count -ne 1 -or $installers[0].Name -ne $expectedName) {
        throw "Expected exactly one Windows installer named $expectedName."
    }

    Write-Host '--- :mag: Verify the release installer'
    Invoke-Checked { & $env:SIGNTOOL_PATH verify /pa /all /v /tw $installers[0].FullName }
} finally {
    Remove-Item $signingConfig -ErrorAction SilentlyContinue
}

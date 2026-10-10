$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Set-Location $repoRoot
$config = Get-Content 'harper-desktop/src-tauri/tauri.conf.json' -Raw | ConvertFrom-Json
if ($env:BUILDKITE_TAG -and $env:BUILDKITE_TAG -ne "v$($config.version)") {
    throw "Release tag must match the configured app version v$($config.version)."
}

Write-Host '--- :rust: Prepare Rust and build tools'
$cargoBin = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME 'bin' } else { Join-Path $env:USERPROFILE '.cargo/bin' }
$env:PATH = "$cargoBin;$env:PATH"
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    $rustupInstaller = Join-Path $env:TEMP 'harper-rustup-init.exe'
    Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile $rustupInstaller
    & $rustupInstaller -y --no-modify-path --default-toolchain stable --profile minimal
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
rustup update stable --no-self-update
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
rustup target add wasm32-unknown-unknown x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not (Get-Command cargo-binstall -ErrorAction SilentlyContinue)) {
    cargo install cargo-binstall --locked
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
cargo binstall --no-confirm --force just wasm-pack
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm install --global pnpm@10.10.0
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$pnpmPrefix = npm prefix --global
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$env:PATH = "$pnpmPrefix;$env:PATH"

Write-Host '--- :lock: Set up Azure Trusted Signing'
$setupScript = (Get-Command setup_azure_trusted_signing.ps1 -ErrorAction Stop).Source
& $setupScript
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# NSIS can invoke the signing hook from a different working directory.
# Absolute paths also avoid parsing a shell command containing spaces.
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
    just build-desktop-windows-signed $signingConfig
    if ($LASTEXITCODE -ne 0) { throw "Windows build failed with exit code $LASTEXITCODE." }

    $bundleDir = 'harper-desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis'
    $installers = @(Get-ChildItem $bundleDir -Filter '*-setup.exe' -File)
    $expectedName = "Harper_$($config.version)_x64-setup.exe"
    if ($installers.Count -ne 1 -or $installers[0].Name -ne $expectedName) {
        throw "Expected exactly one Windows installer named $expectedName."
    }

    Write-Host '--- :mag: Verify the release installer'
    & $env:SIGNTOOL_PATH verify /pa /all /v /tw $installers[0].FullName
    if ($LASTEXITCODE -ne 0) { throw 'Windows installer signature verification failed.' }
} finally {
    Remove-Item $signingConfig -ErrorAction SilentlyContinue
}

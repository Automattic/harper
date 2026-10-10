---
title: Harper Desktop
---

Harper Desktop aims to (eventually) serve the needs of users on a variety of platforms.
One day, we hope for it to include support for macOS, Windows, and Linux.
Right now, it supports macOS and Windows.
Because we hope to add new platforms, there are structures in the code that would not make sense for an app specifically designed with only one operating system in mind.

This document will serve to give a tour of how the code for the Desktop app is structured and, more importantly, why it is structured that way.

## The Processes

When running, the Harper Desktop app will start two distinct processes.
I will give a brief overview here, then go into more detail in later sections.

The first is a conventional [Tauri](https://tauri.app/) app, with an event loop we do not have control over.
When running the Harper Desktop binary with no arguments, this "main" process is what is kicked off.
If you have not worked with a Tauri app before, I suggest you take a look at their documentation for more details.
In this main process, we try to follow their conventions whenever possible.

When the "Harper Service" is started (which happens when the main process starts, unless the user has disabled it) the main process will kick off a second "highlighter" process by running the same binary with the `highlighter` argument.
This second process is what actually reads and writes text from user applications (using a platform's Accessibility API), performs linting, and renders highlights and suggestion popups over the top of their screen.
We need to separate this highlighter service process from the main process because it needs to maintain its own custom event loop, which was not possible with Tauri.
It also needs to be able to place itself in special "modes" within the operating system, which allow it to open windows without taskbar icons or frames.

## The Main Process

As I said above, the main process is a conventional Tauri app.
It holds one primary responsibility, which is to manage configuration and state for the rest of the app.
Everything else it does is supplemental.
That means that it:

- Loads configuration from disk at startup.
- Saves configuration to disk whenever it changes.
- Exposes UI for adjusting the configuration to the user's preference.

The most important module in the Desktop app is likely `config`, which contains a central record of the entire application's state.
Much of the "fluff" code in the project serves to shuffle copies of this config to various parts of the app, including, but not limited to, the settings page WebView, the highlighter process, and to disk.

## The Highlighter Process

This process is much more interesting.
To reiterate: this process's job is to read text from the screen, lint it, and display highlights and suggestion popups.
There are three main components to this:

- Its communication with the main thread.
- Its communication with the operating system via the accessibility API.
- Its communication with the user via an invisible window which it renders on top of every other window on the user's desktop.

It communicates with the main process using the protocol defined in the `communication` module.
It does so to receive updates to its internal lint configuration so that it matches the main process's state as closely as possible.
When the user changes a setting via a Tauri window, that setting is propagated from the WebView to the main process, which is then shared to the highlighter.
The latter step happens in the `communication` module.

The communication with the operating system happens inside of a `Broker`, which is an interface that exposes any and all platform-specific functions Harper needs.
The platform implementations are `MacBroker` and `WindowsBroker`.

Finally, the communication with the user happens via a `winit` window, to which we render [`egui`](https://github.com/emilk/egui) elements.
The exact structure of the relevant modules for this is likely subject to change and may vary by platform, so I will forgo including additional information here to avoid misleading you.

I want to emphasize this point: the highlighter process does not store any canonical state. Any updates to its state must be synchronized to the main process as soon as possible.

## In the Monorepo

Harper Desktop resides in the Harper monorepo, but is not included in the `cargo` workspace. Why? It is because the Desktop app includes (through Tauri) far too many dependencies.
Small changes in unrelated crates would often cause large rebuilds and rechecks.
To solve it, we simply removed it from the dependency graph.

## Windows Builds and Signing

Buildkite builds and signs the Windows x64 NSIS installer on Automattic's Windows agents.
GitHub Actions also cross-compiles an unsigned installer on Linux for build checks, including fork pull requests.
Only Buildkite uploads Windows installers to GitHub Releases.

The unsigned Linux build remains available through:

```sh
just build-desktop-windows
```

The signed native build uses `.buildkite/commands/build-desktop-windows.ps1`.
It prepares Rust, Just, wasm-pack, and pnpm, then calls the shared CI Toolkit's `setup_azure_trusted_signing.ps1`, as Studio does.
The Windows CI image supplies Git Bash and the Visual Studio C++ build tools.
The script generates a temporary Tauri signing configuration and passes it to `just build-desktop-windows-signed <config-path>`.
Local unsigned builds do not need signing credentials or this configuration.

### CI Environment

The Windows job needs these values from Automattic's CI secret infrastructure:

- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`
- `AZURE_ENDPOINT`
- `AZURE_CODE_SIGNING_ACCOUNT`
- `AZURE_CERTIFICATE_PROFILE`

Use the existing Automattic signing account and certificate profile.
The CI Toolkit setup supplies `SIGNTOOL_PATH`, `AZURE_CODE_SIGNING_DLIB`, `AZURE_METADATA_JSON`, and the timestamp settings.
Missing signing configuration or a failed setup smoke test stops the build.
The separate upload job runs on the macOS queue using Harper's existing `GITHUB_TOKEN`; the Windows build does not need a release-upload token.
Buildkite's pipeline configuration excludes fork builds.

### What Gets Signed

Tauri invokes `.buildkite/commands/sign-windows.cjs` while packaging the application, uninstaller, bundled signable dependencies, and final installer.
The hook uses SHA256, requests a timestamp, and verifies each signature before returning.
Signing only the finished installer would leave the installed application unsigned.

Verification happens inside the hook because Tauri restores the original application binary after packaging.
Inspecting that restored build output would not verify the executable embedded in the installer.
The build script also verifies the final installer before succeeding.

Authenticode signing identifies the publisher to Windows.
It is separate from Tauri's updater signatures, which authenticate update downloads.
Windows updater artifacts remain disabled, and the website's updater endpoint currently serves macOS only.
This workflow preserves the existing macOS updater artifacts.
Signing does not guarantee that a new release will immediately avoid all SmartScreen reputation warnings.

### Release Uploads and Verification

For version tags, Buildkite uploads `Harper_<version>_x64-setup.exe` to the matching draft GitHub release.
The tag must match the app version in `tauri.conf.json`.
The upload step waits for the successful Windows build and the macOS release step, then downloads the artifact from that Windows step in the current build.
It reuses the draft, preserves notes and unrelated assets, and replaces only matching filenames when retried.
Uploads to already-published releases fail; this workflow does not publish releases.

Run the tests for signing and release handling without credentials:

```sh
bundle install
just test-desktop-release
```

These tests also run in the macOS Buildkite job.
Before distributing the first signed Windows release, run the Windows Buildkite job with the configured signing environment.
Check the signing and verification logs, install the artifact on Windows, and confirm that the installer, installed Harper executable, and uninstaller have valid Automattic signatures.
Then launch Harper and exercise uninstall.
Automated local tests cannot establish Azure access or replace that installation check.

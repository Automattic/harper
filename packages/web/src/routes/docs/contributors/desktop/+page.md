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

Buildkite builds and signs the Windows x64 NSIS installer; only Buildkite uploads it to GitHub Releases.
GitHub Actions runs unsigned Linux cross-builds (`just build-desktop-windows`), including fork PRs. Buildkite excludes forks.

`.buildkite/commands/build-desktop-windows.ps1` prepares the tools and CI Toolkit's Azure signing setup, then calls `just build-desktop-windows-signed <config-path>` with a temporary Tauri config.
Windows agents need Git Bash and Visual Studio C++ build tools.

### CI Setup

Supply these values through CI secrets, using Automattic's existing signing account and certificate profile:

- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`
- `AZURE_ENDPOINT`
- `AZURE_CODE_SIGNING_ACCOUNT`
- `AZURE_CERTIFICATE_PROFILE`

The CI Toolkit supplies SignTool and its configuration.
The upload job uses the macOS queue's existing `GITHUB_TOKEN`.

Tauri's `sign-windows.cjs` hook signs and verifies the app, uninstaller, bundled signable dependencies, and installer with SHA256 and a timestamp.
Authenticode signing is separate from updater signing; Windows updater artifacts remain disabled.
SmartScreen may still show reputation warnings.

### Releases and Verification

For `v<version>` tags, the Windows build fails early unless the tag matches `tauri.conf.json`.
The upload step waits for the Windows build to succeed and for the macOS step to finish, then downloads `Harper_<version>_x64-setup.exe` from the Windows build and attaches it to the tag's draft release.
Retrying the upload replaces the installer in the draft and leaves notes and other assets alone; it does not rebuild or re-sign.
Retry the upload step rather than rebuilding the tag: a rebuild runs the macOS step again, which creates another draft, and the upload then refuses to choose between two drafts holding a `.dmg`.

The uploader never creates a release and never modifies a published one.
When several drafts exist for the tag, it uses the one holding the macOS `.dmg`, which the macOS step creates earlier in the same build.
If it fails because no draft exists yet, retry it once the macOS or GitHub Actions jobs have created one.
If it fails because no single draft holds the `.dmg`, reconcile the drafts on GitHub and retry.

The GitHub Actions workflows can leave further drafts for the tag even when the Windows upload succeeds.
Before publishing, wait for the GitHub Actions workflows and the Buildkite pipeline to finish, move the assets of any other draft for the tag into the one you publish, delete the emptied drafts, and confirm that the draft holds the Windows installer with the same SHA256 as the Buildkite artifact.

Run `just test-desktop-release` for the signing and upload tests; GitHub Actions runs them on every pull request.

### Windows Smoke Test

Download the signed installer from the chosen commit's Buildkite build. On Windows:

1. Check the installer's signature with `Get-AuthenticodeSignature .\Harper_<version>_x64-setup.exe`. Expect `Status: Valid` and an Automattic signer.
2. Install Harper, launch it, and check that settings open and the highlighter starts.
3. In the installation directory, check `harper-desktop.exe` and `uninstall.exe` with `Get-AuthenticodeSignature`. Both must have valid Automattic signatures.
4. Quit Harper from its tray menu, uninstall it, and confirm that its installation directory is removed.

Record the build number and smoke-test result with the release's testing evidence, or in the PR when validating a signing change. Build-time signature verification does not replace this installed-app check.

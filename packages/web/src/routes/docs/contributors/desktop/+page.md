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

Buildkite produces the signed Windows x64 installer using Automattic's certificate; signing or signature-verification failures fail the build.
GitHub Actions also checks an unsigned Windows build, including for fork pull requests.
Windows updater artifacts remain disabled.

### Release Upload

Release tags must match the version in `harper-desktop/src-tauri/tauri.conf.json`.
Buildkite attaches the signed installer to an existing draft release; publishing remains manual.
When several drafts exist for the tag, it uses the one holding the macOS `.dmg`.

If the upload fails, retry only the Windows upload step after resolving the error.
Retrying replaces the Windows installer without changing release notes or other assets.
Avoid rebuilding the whole tag, which can create another draft.

Before publishing, ensure the chosen draft holds the signed Windows installer, the macOS build, and the other expected assets.
If several drafts exist, move their assets and notes into the one you publish and delete the rest.
Verify that the Windows installer's SHA256 matches the Buildkite artifact.

Run `just test-desktop-release` to check the signing and upload scripts.

### Windows Smoke Test

Before a release, or after changing the signing setup, download the installer from the Buildkite build and check it on Windows:

1. Check the installer with `Get-AuthenticodeSignature`. Expect `Status: Valid` and an Automattic signer.
2. Install and launch Harper, open the settings, and check that the highlighter starts.
3. Check `harper-desktop.exe` and `uninstall.exe` in the installation directory the same way.
4. Quit Harper from the tray menu, uninstall it, and confirm that the installation directory is gone.

Build-time verification does not replace this check of the installed app.

---
title: JetBrains IDEs
---

You can use [`harper-ls`](./language-server) in any IDE built on the IntelliJ Platform — IntelliJ IDEA, WebStorm, PyCharm, Rider, GoLand, RustRover, CLion, PhpStorm, RubyMine, and more — through the community [LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij) plugin, which ships with `harper-ls` support built in. No Harper-specific plugin is required.

## Required Setup

Make sure you have `harper-ls` installed and available on your `PATH`. You can do this using the [supported installation methods](./language-server#Installation).

Install [LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij) from `Settings/Preferences > Plugins > Marketplace`, then search for "LSP4IJ" and click "Install".

## Enabling `harper-ls`

LSP4IJ includes a predefined entry for `harper-ls` under `Settings/Preferences > Languages & Frameworks > Language Servers`. Enable it there, or add `harper-ls` as a user-defined language server pointing at your installed binary if your version of LSP4IJ doesn't list it yet.

For the exact, up-to-date steps and screenshots, see [LSP4IJ's `harper-ls` documentation](https://github.com/redhat-developer/lsp4ij/blob/main/docs/user-defined-ls/harper-ls.md), since that page is maintained by the LSP4IJ project and may change independently of Harper.

## Configuration

`harper-ls` is configured the same way regardless of editor. See the [configuration section](./language-server#Configuration) of our `harper-ls` documentation for the full list of options, including dialect, dictionaries, and which linters to enable.

---
title: Visual Studio Code
---

For our Visual Studio Code integration, we provide an extension powered by [`harper-ls`](./language-server), which also works for VS Code forks like VSCodium and Windsurf. It's available in the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper) as well as the [Open VSX Registry](https://open-vsx.org/extension/elijah-potter/harper).

## Installation

Open the Extensions view in your editor by selecting the Extensions icon in the Activity Bar or by using the `Ctrl+Shift+X` keyboard shortcut, then search for "Harper" and click "Install".

## Commands

| Command                         | ID                              | Description          |
| ------------------------------- | ------------------------------- | -------------------- |
| Harper: Restart Language Server | `harper.languageserver.restart` | Restarts `harper-ls` |

## Settings

<!-- TODO: Link to the `harper-ls` configuration section here when it's done. -->

The settings below are VS Code specific. You can head over to the `harper-ls` documentation to view other available settings.

| Setting       | Type     | Default Value | Description                                                                                                                                                 |
| ------------- | -------- | ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `harper.path` | `string` | `""`          | Optional path to a `harper-ls` executable to use. Primarily useful if the bundled binary doesn't work in your system like in immutable Linux distributions. |

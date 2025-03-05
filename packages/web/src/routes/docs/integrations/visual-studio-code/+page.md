---
title: Visual Studio Code
---

For our Visual Studio Code integration, we provide an extension powered by [`harper-ls`](./language-server), which also works for VS Code forks like VSCodium and Windsurf. It's available in the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper) as well as the [Open VSX Registry](https://open-vsx.org/extension/elijah-potter/harper).

## Installation

Open the Extensions view in your editor by selecting the Extensions icon in the Activity Bar or by using the `Ctrl+Shift+X` keyboard shortcut, then search for "Harper" and click "Install".

If you prefer to use the command line, you can use the following command:

```bash
code --install-extension elijah-potter.harper
```

## Commands

| Command                         | ID                              | Description          |
| ------------------------------- | ------------------------------- | -------------------- |
| Harper: Restart Language Server | `harper.languageserver.restart` | Restarts `harper-ls` |

## Settings

The settings below are VS Code specific. There are other settings that `harper-ls` supports such as which linters to use or how code actions should appear that you can configure. You can view them in your editor's Settings UI under "Harper" or peruse through them in the [configuration section](./language-server#Configuration) of our `harper-ls` documentation.

| Setting       | Type     | Default Value | Description                                                                                                                                                 |
| ------------- | -------- | ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `harper.path` | `string` | `""`          | Optional path to a `harper-ls` executable to use. Primarily useful if the bundled binary doesn't work in your system like in immutable Linux distributions. |

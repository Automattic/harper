---
title: Visual Studio Code
---

This document details how to develop the Visual Studio Code extension locally. If you're interested in how it's packaged and distributed, you can checkout the [Release VS Code Plugin](https://github.com/Automattic/harper/blob/master/.github/workflows/release_vscode_plugin.yml) workflow.

## Notes

- The extension code and its tests live in the `src` directory. Most changes you'll need to make will be there.
- VS Code can only pickup the tasks and launch configurations set in `packages/vscode-plugin/.vscode` if this directory, `packages/vscode-plugin`, not the root of the Harper repository, is open.
- You can look at the project's [`justfile`](https://github.com/Automattic/harper/blob/master/justfile) to see exactly what running the `just` recipes below do.

## Prerequisites

- Make sure to [set up your environment](./environment).
- Before running or testing the extension using VS Code's Debugger, make sure you have `harper-ls` in `packages/vscode-plugin/bin`. You can either manually create the directory, compile `harper-ls`, and put it there or you can run `just test-vscode` or `just package-vscode` which will do that for you.

## Running the Extension

1. Open the Run and Debug view by selecting it from the Activity Bar or by pressing `Ctrl+Shift+D`.
2. Choose `Run Extension`, if not chosen already.
3. Click the play (Start Debugging) button or press `F5`.

## Running the Tests

### Using VS Code's Debugger

1. Open the Run and Debug view by selecting it from the Activity Bar or by pressing `Ctrl+Shift+D`.
2. Choose `Test Extension`, if not chosen already.
3. Click the play (Start Debugging) button or press `F5`.

### Using the Command Line

```bash
just test-vscode
```

## Packaging and Installing the Extension

1. Package the extension:

   ```bash
   just package-vscode
   ```

2. Install the extension:

   ```bash
   code --install-extension path/to/created/.vsix
   ```

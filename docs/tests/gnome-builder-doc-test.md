# Test: GNOME Builder documentation

This file validates the structure and content of `docs/gnome-builder.md`.

## Checks

- [x] File exists at `docs/gnome-builder.md`
- [x] Contains installation instructions (`cargo install harper-ls`)
- [x] Contains plugin directory path (`~/.local/share/gnome-builder/plugins/`)
- [x] Contains `.plugin` file example with required fields (Module, Loader, Name)
- [x] Contains Python plugin source with `Ide.LspService` subclass
- [x] Registers `harper-ls --stdio` as the command
- [x] Registers at least `markdown` language
- [x] Contains restart instructions

## Edge cases

- [x] Plugin descriptor uses `python3` loader (not `python`)
- [x] Python source uses `gi.require_version` before importing modules
- [x] `do_configure_launcher` uses `--stdio` flag (required for LSP stdio transport)

# Using Harper with GNOME Builder

GNOME Builder has built-in support for the Language Server Protocol (LSP). You can configure it to use `harper-ls` by creating a small plugin.

## Install harper-ls

```bash
cargo install harper-ls --locked
```

Ensure `harper-ls` is available on your `$PATH`.

## Create the plugin directory

```bash
mkdir -p ~/.local/share/gnome-builder/plugins/harper-ls
```

## Create the plugin descriptor

Create `~/.local/share/gnome-builder/plugins/harper-ls/harper_ls.plugin`:

```ini
[Plugin]
Module=harper_ls
Loader=python3
Name=Harper LS
Description=Harper grammar checker via harper-ls
Authors=Harper contributors
X-Category=lsps
```

## Create the plugin source

Create `~/.local/share/gnome-builder/plugins/harper-ls/harper_ls.py`:

```python
import gi

gi.require_version("Ide", "1.0")
gi.require_version("GtkSource", "5")

from gi.repository import GObject, Gio, GtkSource, Ide


class HarperLspService(Ide.LspService):
    class Meta:
        id = "harper-ls"

    def do_configure_launcher(self, pipeline, launcher):
        launcher.set_argv(["harper-ls", "--stdio"])

    def do_configure_client(self, client):
        client.add_language("markdown")
        client.add_language("plaintext")


class HarperDiagnosticProvider(Ide.LspDiagnosticProvider, Ide.DiagnosticProvider):
    def do_load(self):
        context = self.get_context()
        service = Ide.LspService.from_context(HarperLspService, context)
        service.attach(self)


class HarperCompletionProvider(
    Ide.LspCompletionProvider, GtkSource.CompletionProvider
):
    def do_load(self, context):
        service = Ide.LspService.from_context(HarperLspService, context)
        service.attach(self)
```

## Restart GNOME Builder

After creating both files, restart GNOME Builder. The plugin will start `harper-ls` automatically when you open Markdown or plain text files.

## Supported file types

The example above registers `markdown` and `plaintext`. To add more languages (for example, `latex` or `html`), add additional `client.add_language("...")` calls in `do_configure_client`.

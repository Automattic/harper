import gi

gi.require_version("Ide", "1.0")
gi.require_version("GObject", "2.0")
gi.require_version("Gio", "2.0")

from gi.repository import GObject, Gio, Ide


class HarperService(Ide.LspService):
    class Meta:
        id = "org.harper.lsp"

    def do_configure_launcher(self, pipeline, launcher):
        launcher.set_cwd(Ide.BuildPipeline.get_srcdir(pipeline))
        launcher.push_argv("harper-ls")
        launcher.push_argv("--stdio")

    def do_configure_client(self, client):
        client.add_language("markdown")
        client.add_language("plain")
        client.add_language("rust")
        client.add_language("python")
        client.add_language("javascript")
        client.add_language("typescript")
        client.add_language("html")
        client.add_language("c")
        client.add_language("cpp")
        client.add_language("go")
        client.add_language("ruby")


class HarperDiagnosticProvider(Ide.LspDiagnosticProvider, Ide.DiagnosticProvider):
    def do_load(self):
        context = self.get_context()
        service = Ide.LspService.from_context(HarperService, context)
        service.attach(self)


class HarperCompletionProvider(Ide.LspCompletionProvider, Ide.CompletionProvider):
    def do_load(self, context):
        service = Ide.LspService.from_context(HarperService, context)
        service.attach(self)


class HarperCodeActionProvider(Ide.LspCodeActionProvider, Ide.CodeActionProvider):
    def do_load(self):
        context = self.get_context()
        service = Ide.LspService.from_context(HarperService, context)
        service.attach(self)

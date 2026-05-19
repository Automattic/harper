import sys
import unittest
from unittest.mock import MagicMock, patch


module_mock = MagicMock()
sys.modules["gi"] = module_mock
sys.modules["gi.repository"] = module_mock.repository


class FakeLspService:
    pass


class FakeDiagnosticProvider:
    pass


class FakeCompletionProvider:
    pass


class FakeCodeActionProvider:
    pass


module_mock.require_version = MagicMock()
module_mock.repository.GObject = MagicMock()
module_mock.repository.Gio = MagicMock()
module_mock.repository.Ide = MagicMock()
module_mock.repository.Ide.LspService = FakeLspService
module_mock.repository.Ide.DiagnosticProvider = FakeDiagnosticProvider
module_mock.repository.Ide.LspDiagnosticProvider = FakeDiagnosticProvider
module_mock.repository.Ide.CompletionProvider = FakeCompletionProvider
module_mock.repository.Ide.LspCompletionProvider = FakeCompletionProvider
module_mock.repository.Ide.CodeActionProvider = FakeCodeActionProvider
module_mock.repository.Ide.LspCodeActionProvider = FakeCodeActionProvider
module_mock.repository.Ide.BuildPipeline = MagicMock()
module_mock.repository.Ide.LspService.from_context = MagicMock()

import importlib
import importlib.util
import os

PLUGIN_PATH = os.path.join(os.path.dirname(__file__), "harper_ls.py")


def load_plugin():
    spec = importlib.util.spec_from_file_location("harper_ls", PLUGIN_PATH)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class TestHarperService(unittest.TestCase):
    def setUp(self):
        self.mod = load_plugin()

    def test_service_class_exists(self):
        self.assertTrue(hasattr(self.mod, "HarperService"))

    def test_diagnostic_provider_class_exists(self):
        self.assertTrue(hasattr(self.mod, "HarperDiagnosticProvider"))

    def test_completion_provider_class_exists(self):
        self.assertTrue(hasattr(self.mod, "HarperCompletionProvider"))

    def test_code_action_provider_class_exists(self):
        self.assertTrue(hasattr(self.mod, "HarperCodeActionProvider"))

    def test_configure_launcher_pushes_correct_args(self):
        service = self.mod.HarperService()
        pipeline = MagicMock()
        launcher = MagicMock()
        module_mock.repository.Ide.BuildPipeline.get_srcdir.return_value = "/src"
        service.do_configure_launcher(pipeline, launcher)
        launcher.set_cwd.assert_called_once_with("/src")
        launcher.push_argv.assert_any_call("harper-ls")
        launcher.push_argv.assert_any_call("--stdio")

    def test_configure_client_adds_languages(self):
        service = self.mod.HarperService()
        client = MagicMock()
        service.do_configure_client(client)
        expected_languages = [
            "markdown", "plain", "rust", "python",
            "javascript", "typescript", "html", "c",
            "cpp", "go", "ruby",
        ]
        for lang in expected_languages:
            client.add_language.assert_any_call(lang)
        self.assertEqual(client.add_language.call_count, len(expected_languages))

    def test_configure_launcher_uses_stdio_flag(self):
        service = self.mod.HarperService()
        pipeline = MagicMock()
        launcher = MagicMock()
        module_mock.repository.Ide.BuildPipeline.get_srcdir.return_value = "/tmp"
        service.do_configure_launcher(pipeline, launcher)
        args = [call[0][0] for call in launcher.push_argv.call_args_list]
        self.assertIn("--stdio", args)


class TestPluginFile(unittest.TestCase):
    def test_plugin_file_exists(self):
        plugin_path = os.path.join(
            os.path.dirname(__file__), "harper_ls.plugin"
        )
        self.assertTrue(os.path.isfile(plugin_path))

    def test_plugin_file_contains_module_name(self):
        plugin_path = os.path.join(
            os.path.dirname(__file__), "harper_ls.plugin"
        )
        with open(plugin_path) as f:
            content = f.read()
        self.assertIn("Module=harper_ls", content)
        self.assertIn("Loader=python3", content)


if __name__ == "__main__":
    unittest.main()

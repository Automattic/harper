import configparser
import os
import unittest


class TestHarperLsPlugin(unittest.TestCase):
	def setUp(self):
		plugin_path = os.path.join(
			os.path.dirname(__file__), "harper_ls.plugin"
		)
		self.config = configparser.ConfigParser()
		self.config.read(plugin_path)

	def test_plugin_section_exists(self):
		self.assertIn("Plugin", self.config.sections())

	def test_module_is_harper_ls(self):
		self.assertEqual(self.config.get("Plugin", "Module"), "harper_ls")

	def test_name_is_set(self):
		name = self.config.get("Plugin", "Name")
		self.assertTrue(len(name) > 0)

	def test_description_is_set(self):
		description = self.config.get("Plugin", "Description")
		self.assertTrue(len(description) > 0)

	def test_language_server_command_uses_stdio(self):
		cmd = self.config.get("Plugin", "X-Language-Server-Command")
		self.assertIn("harper-ls", cmd)
		self.assertIn("--stdio", cmd)

	def test_language_server_languages_not_empty(self):
		langs = self.config.get("Plugin", "X-Language-Server-Languages")
		self.assertTrue(len(langs) > 0)

	def test_content_type_not_empty(self):
		content_type = self.config.get("Plugin", "X-Content-Type")
		self.assertTrue(len(content_type) > 0)

	def test_no_extra_sections(self):
		self.assertEqual(self.config.sections(), ["Plugin"])


if __name__ == "__main__":
	unittest.main()

import argparse
import base64
import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import passive_voice_corpus as corpus


class CorpusTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name) / "corpus"
        self.output = Path(self.temp.name) / "report.json"

    def collect(self):
        responses = [{"items": [{"full_name": "owner/project", "default_branch": "main"}]},
                     {"sha": "abc123"},
                     {"size": 8, "encoding": "base64", "content": base64.b64encode(b"Example.").decode(),
                      "path": "README.md"}]
        with patch.object(corpus, "api", side_effect=responses) as api, contextlib.redirect_stdout(io.StringIO()):
            corpus.collect(argparse.Namespace(directory=self.directory, query="stars:>=500", seed=0, limit=1))
        self.assertEqual(api.call_args_list[-1].args[0], "repos/owner/project/readme?ref=abc123")

    def review_args(self):
        return argparse.Namespace(directory=self.directory, output=self.output, harper=Path("harper-cli"))

    def test_private_repositories_are_not_downloaded(self):
        response = {"items": [{"full_name": "owner/private", "private": True}]}
        with patch.object(corpus, "api", return_value=response) as api, contextlib.redirect_stdout(io.StringIO()):
            corpus.collect(argparse.Namespace(directory=self.directory, query="stars:>=500", seed=0, limit=1))
        api.assert_called_once()
        manifest = json.loads((self.directory / "manifest.json").read_text())
        self.assertEqual(manifest["documents"], [])

    def test_pinned_source_and_nonzero_lint_exit(self):
        self.collect()
        result = [{"lint_count": 1, "lints": [{"matched_text": "was reviewed"}]}]
        run = subprocess.CompletedProcess([], 1, stdout=json.dumps(result))
        with patch.object(corpus.subprocess, "run", return_value=run), contextlib.redirect_stdout(io.StringIO()):
            corpus.review(self.review_args())
        report = json.loads(self.output.read_text())
        self.assertEqual(report["documents"][0]["source"], "https://github.com/owner/project/blob/abc123/README.md")
        self.assertEqual(report["documents"][0]["result"], result[0])

    def test_changed_corpus_is_rejected_before_linting(self):
        self.collect()
        (self.directory / "owner__project.md").write_text("Changed")
        with patch.object(corpus.subprocess, "run") as run, self.assertRaises(ValueError):
            corpus.review(self.review_args())
        run.assert_not_called()

    def test_cli_error_is_not_reported_as_zero_findings(self):
        self.collect()
        run = subprocess.CompletedProcess([], 1, stdout='[{"error": "Cannot read input"}]')
        with patch.object(corpus.subprocess, "run", return_value=run), self.assertRaises(ValueError):
            corpus.review(self.review_args())
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()

# Passive voice corpus review

This optional tool collects up to 100 public GitHub project READMEs using Python 3 and
an authenticated `gh` CLI. It records immutable commit URLs, SHA-256 hashes,
and repository license metadata. Collection is sequential; errors remain in the
manifest. The seed chooses an order within the first 100 search results, which
are sorted by stars. This is a convenience sample of popular projects, not an
unbiased sample or an English-only dataset. Search results change over time;
keep the downloaded files and manifest to reproduce a run.

From the repository root:

```sh
python3 tools/passive_voice_corpus.py collect /tmp/harper-readmes --limit 100
cargo build -p harper-cli
python3 tools/passive_voice_corpus.py review /tmp/harper-readmes --output /tmp/passive-review.json
```

The review command works offline with the cached corpus. It verifies file hashes
and runs only PassiveVoice, with JSON output, sequential processing, and empty
user/file dictionaries. Keep the same Harper executable for comparisons. Each
result includes its original source URL. Inspect the matched phrases and their
surrounding context manually before promoting excerpts to regression tests.
Findings are unlabelled candidates, not false positives or an accuracy metric.
The rule is opt-in and English-focused; foreign-language prose needs separate
human judgment.

Downloads and reports belong outside the repository. License metadata describes
the project, and may not describe every README contribution. Check the source's
terms before redistributing text; this tool does not grant permission. Commit
only short, attributed regression excerpts when warranted.

Offline tool checks: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p "test_*.py"`.

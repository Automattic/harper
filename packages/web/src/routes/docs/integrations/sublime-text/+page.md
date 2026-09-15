---
title: Sublime Text
---

Our [Sublime Text](https://www.sublimetext.com/) integration is powered by [`harper-ls`](./language-server).

## Required Setup

Make sure you have `harper-ls` installed and available on your global or Sublime Text's `PATH`. You can do this using the [supported installation methods](./language-server#Installation).

Ensure you have [LSP for Sublime Text](https://lsp.sublimetext.io/) installed.

## Optional Configuration

Open `Preferences > Package Settings > LSP > Settings` and add the `harper-ls` client configuration to the "clients" section:

```json title=LSP.sublime-settings
{
  "clients": {
    "harper-ls": {
      "enabled": true,
      "command": [
        "harper-ls",
        "--stdio"
      ],
      "selector": "source.markdown | text.html.markdown | text.plain",
      "settings": {
        "harper-ls": {
          "userDictPath": "",
          "workspaceDictPath": "",
          "fileDictPath": "",
          "linters": {
            "SpellCheck": true,
            "SpelledNumbers": false,
            "AnA": true,
            "SentenceCapitalization": true,
            "UnclosedQuotes": true,
            "WrongApostrophe": false,
            "LongSentences": true,
            "RepeatedWords": true,
            "Spaces": true,
            "CorrectNumberSuffix": true
          },
          "codeActions": {
            "ForceStable": false
          },
          "markdown": {
            "IgnoreLinkTitle": false
          },
          "diagnosticSeverity": "hint",
          "isolateEnglish": false,
          "dialect": "American",
          "maxFileLength": 120000,
          "ignoredLintsPath": "",
          "excludePatterns": []
        }
      }
    }
  }
}
```

For more information on what each of these configs do, you can head over to the [configuration section](./language-server#Configuration) of our `harper-ls` documentation.

## Linting Git Commit Messages

Commit messages need two extra pieces of configuration in Sublime Text.

First, add the commit message scope to the `selector` in your `harper-ls` client configuration above, so LSP attaches to commit buffers at all:

```json title=LSP.sublime-settings
"selector": "source.markdown | text.html.markdown | text.plain | text.git.commit",
```

Second, tell Sublime which language ID to report for that scope. By default Sublime sends `git` for everything in its `text.git.*` family, and `harper-ls` routes commit messages by the `git-commit` language ID, so the buffer goes unlinted. Add the mapping to your user `language-ids.sublime-settings`, which LSP for Sublime Text reads to override the language ID it derives from a scope:

```json title=language-ids.sublime-settings
{
  "text.git.commit": "git-commit"
}
```

With both in place, `git commit` buffers opened in Sublime Text are checked with the same Git commit parser used by the other editor integrations.

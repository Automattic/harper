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
            "WrongQuotes": false,
            "LongSentences": true,
            "RepeatedWords": true,
            "Spaces": true,
            "Matcher": true,
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
          "dialect": "American"
        }
      }
    }
  }
}
```

For more information on what each of these configs do, you can head over to the [configuration section](./language-server#Configuration) of our `harper-ls` documentation.

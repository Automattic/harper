---
title: Helix
---

Our Helix integration is powered by [`harper-ls`](./language-server).

## Required Setup

Make sure you have [`harper-ls` installed](./language-server#Installation) on your system and available in your `PATH`.

Helix supports language servers [out-of-the-box](https://docs.helix-editor.com/languages.html), but you'll still need to configure it to use `harper-ls`. First, you need to tell Helix how it should run `harper-ls`:

```toml title=languages.toml
[language-server.harper-ls]
command = "harper-ls"
args = ["--stdio"]
```

Then, for all the [languages `harper-ls` supports](./language-server#Supported-Languages) that you want it to be enabled for, you need to declare the following in your `languages.toml`:

```toml title=languages.toml
[[language]]
name = "language-id"
language-servers = ["default-servers", "harper-ls"]
```

where `language-id` is the language ID of the language you want `harper-ls` to be used for and `default-servers` are any of the [default language servers](https://docs.helix-editor.com/lang-support.html) supported by Helix that you use for that language. For example, if you want to configure it for Markdown and you use both Marksman and Markdown-Oxide, you'd end up with this:

```toml title=languages.toml
[[language]]
name = "markdown"
language-servers = ["marksman", "markdown-oxide", "harper-ls"]
```

You need to include the default language servers since there currently isn't a way to append a language server to the default `language-servers` list. Of course, you can also add other language servers you use before or after `harper-ls`.

## Optional Configuration

Additionally, you can also configure things like which linters to use or how you want code actions to appear. Below is an example config where everything is set to their default values:

```toml title=languages.toml
[language-server.harper-ls.config.harper-ls]
userDictPath = ""
workspaceDictPath = ""
fileDictPath = ""
diagnosticSeverity = "hint"
isolateEnglish = false
dialect = "American"
maxFileLength = 120000

[language-server.harper-ls.config.harper-ls.linters]
SpellCheck = true
SpelledNumbers = false
AnA = true
SentenceCapitalization = true
UnclosedQuotes = true
WrongQuotes = false
LongSentences = true
RepeatedWords = true
Spaces = true
Matcher = true
CorrectNumberSuffix = true

[language-server.harper-ls.config.harper-ls.codeActions]
ForceStable = false

[language-server.harper-ls.config.harper-ls.markdown]
IgnoreLinkTitle = false
```

:::note
This example only contains some of the available linters, check out our [rules page](../rules) to view the full list.
:::

For more information on what each of these configs do, you can head over to the [configuration section](./language-server#Configuration) of our `harper-ls` documentation.

## Common Config Changes

Programmers often find certain rules have too much of a hair-trigger.
The below config is a simple cut-and-paste that gives you much fewer false-positives.

```toml title=languages.toml
[language-server.harper-ls.config.harper-ls.linters]
SpellCheck = false
SentenceCapitalization = false
```

## Additional Links

- [Helix's official documentation on `harper-ls`](https://github.com/helix-editor/helix/wiki/Language-Server-Configurations#harper-ls)
- [Community discussion on configuring `harper-ls` for Helix](https://github.com/Automattic/harper/discussions/135)

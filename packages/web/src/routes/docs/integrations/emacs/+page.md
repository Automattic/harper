---
title: Emacs
---

Our Emacs integration is powered by [`harper-ls`](./language-server).

## Required Setup

Make sure you have [`harper-ls` installed](./language-server#Installation) on your system and available in your `PATH`.

Since version 29, Emacs has had native support for the Language Server Protocol through [Eglot](https://www.gnu.org/software/emacs/manual/html_mono/eglot.html), so all you have to do is configure it to use `harper-ls` in your `init.el`:

```elisp title=init.el
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(text-mode . ("harper-ls" "--stdio"))))
```

where `text-mode` can be set to any, some, or all major modes that correspond to the [languages `harper-ls` supports](./language-server#Supported-Languages). Typically, if you want to use `harper-ls` to edit Markdown files and you have [`markdown-mode`](https://jblevins.org/projects/markdown-mode) installed, you can configure it like this:

```elisp title=init.el
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(markdown-mode . ("harper-ls" "--stdio"))))
```

:::note

A possible gotcha you may be encountering if you don't see any diagnostics is when Eglot automatically deduces the language ID being sent to `harper-ls` based on the major mode you used. In which case, you can set the language ID to one that is supported:

```elisp title=init.el
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '((english-prose-mode :language-id "plaintext") . ("harper-ls" "--stdio"))
```

:::

## Optional Configuration

Additionally, you can also configure things like which linters to use or how you want code actions to appear. Below is an example config where everything is set to their default values:

```elisp title=init.el
(setq-default eglot-workspace-configuration
              '(:harper-ls (:userDictPath ""
                            :workspaceDictPath ""
                            :fileDictPath ""
                            :linters (:SpellCheck t
                                      :SpelledNumbers :json-false
                                      :AnA t
                                      :SentenceCapitalization t
                                      :UnclosedQuotes t
                                      :WrongQuotes :json-false
                                      :LongSentences t
                                      :RepeatedWords t
                                      :Spaces t
                                      :Matcher t
                                      :CorrectNumberSuffix t)
                            :codeActions (:ForceStable :json-false)
                            :markdown (:IgnoreLinkTitle :json-false)
                            :diagnosticSeverity "hint"
                            :isolateEnglish :json-false
                            :dialect "American"
                            :maxFileLength 120000)))
```

:::note
This example only contains some of the available linters, check out our [rules page](../rules) to view the full list.
:::

For more information on what each of these configs do, you can head over to the [configuration section](./language-server#Configuration) of our `harper-ls` documentation.

## Common Config Changes

Programmers often find certain rules have too much of a hair-trigger.
The below config is a simple cut-and-paste that gives you much fewer false-positives.

```elisp title=init.el
(setq-default eglot-workspace-configuration
              '(:harper-ls (:linters (:SpellCheck :json-false
                                      :SentenceCapitalization :json-false))))
```

## Additional Links

- [Community discussion on configuring `harper-ls` for Emacs](https://github.com/Automattic/harper/discussions/150)

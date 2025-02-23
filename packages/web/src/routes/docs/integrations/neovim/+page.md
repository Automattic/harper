---
title: Neovim
---

Our Neovim integration is powered by [`harper-ls`](./language-server).

## Installation

How you choose to install `harper-ls` depends on your use-case. For Neovim, we only directly support usage through [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#harper_ls) and installation via [`mason.nvim`](https://mason-registry.dev/registry/list?search=harper-ls). However, you can also [install it separately](./language-server#Installation) and set it up yourself if you wish.

## Configuration

Neovim is also one of the two primarily supported editors for `harper-ls`.
As such, you can view this page as canonical documentation for the available configuration options.
[Helix](./helix) and [Zed](./zed) users may also find it helpful.

### Markdown-Specific Config

The Markdown parser has its own configuration option, used to modify its behavior in specific ways.
For example, the title of a link is linted by default, but this behavior can be changed through the `ignore_link_title` key:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      markdown = {
        ignore_link_title = true,
      }
    }
  },
}
```

### Dictionaries

You do not have to stick with the default dictionary locations ([listed on this page](./language-server)).
If you use Neovim, you can set the location of the user dictionary with the `userDictPath` key, and the file dictionary with the `fileDictPath` key:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      userDictPath = "~/dict.txt",
      fileDictPath = "~/.harper/",
    }
  },
}
```

For example, if you want to use Vim's dictionary, you can do something like this:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      userDictPath = vim.fn.stdpath("config") .. "/spell/en.utf-8.add",
    }
  },
}
```

See the [relevant issue for details](https://github.com/Automattic/harper/issues/143).

### Linters

Linters are grammatical rules Harper looks for to correct your work.
You can toggle them on or off to your liking.

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      linters = {
        SpellCheck = true,
        SpelledNumbers = false,
        AnA = true,
        SentenceCapitalization = true,
        UnclosedQuotes = true,
        WrongQuotes = false,
        LongSentences = true,
        RepeatedWords = true,
        Spaces = true,
        Matcher = true,
        CorrectNumberSuffix = true,
      }
    }
  },
}
```

<script>
import DefaultNeovimConfig from "$lib/DefaultNeovimConfig.svelte"
</script>

<DefaultNeovimConfig/>

By default, `harper-ls` will mark all diagnostics with HINT.
If you want to configure this, refer below:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
        diagnosticSeverity = "hint" -- Can also be "information", "warning", or "error"
    }
  },
}
```

You can also configure how `harper-ls` displays code actions.
For example, to make code actions appear in "stable" positions, use the following configuration:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      codeActions = {
        forceStable = true
      }
    }
  },
}
```

If you work with a lot of documents that are a mixture of English and another language, Harper can attempt to automatically detect which is which and only lint the English text.
To enable it, just set the `isolateEnglish` key.

:::note
This feature is incredibly new and unstable.
Do not expect it to work perfectly.
If improvements are important to you, feel free to [open an issue](https://github.com/Automattic/harper/issues/new?template=Blank+issue) to let us know.
:::

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
        isolateEnglish = false
    }
  },
}
```

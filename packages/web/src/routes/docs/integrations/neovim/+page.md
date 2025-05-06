---
title: Neovim
---

Our Neovim integration is powered by [`harper-ls`](./language-server).

## Required Setup

Make sure you have `harper-ls` installed and available on your global or Neovim's `PATH`. You can do this using [`mason.nvim`](https://mason-registry.dev/registry/list?search=harper-ls) or via any of our other [supported installation methods](./language-server#Installation).

Though Neovim supports language servers [out-of-the-box](https://neovim.io/doc/user/lsp.html), for ease of use, we suggest using `harper-ls` through [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig).

Once you have `harper-ls` and nvim-lspconfig installed, you need to add this to your `init.lua`:

```lua title=inti.lua
require('lspconfig').harper_ls.setup {}
```

## Optional Configuration

Additionally, you can also configure things like which linters to use or how you want code actions to appear. Below is an example config where everything is set to their default values:

```lua title=init.lua
require('lspconfig').harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      userDictPath = "",
      fileDictPath = "",
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
        CorrectNumberSuffix = true
      },
      codeActions = {
        ForceStable = false
      },
      markdown = {
        IgnoreLinkTitle = false
      },
      diagnosticSeverity = "hint",
      isolateEnglish = false,
      dialect = "American"
    }
  }
}
```

:::note
This example only contains some of the available linters, check out our [rules page](../rules) to view the full list.
:::

For more information on what each of these configs do, you can head over to the [configuration section](./language-server#Configuration) of our `harper-ls` documentation.

## Common Config Changes

Programmers often find certain rules have too much of a hair-trigger.
The below config is a simple cut-and-paste that gives you much fewer false-positives.

```lua title=init.lua
require('lspconfig').harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      linters = {
        SentenceCapitalization = false,
        SpellCheck = false
      }
    }
  }
}
```

## Additional Links

- [nvim-lspconfig's documentation on `harper-ls`](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#harper_ls)

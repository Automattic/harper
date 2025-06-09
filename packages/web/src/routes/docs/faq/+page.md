---
title: Frequently Asked Questions
---

## How Do I Use or Integrate Harper?

That depends on your use case.

Do you want to use it within Obsidian? We have an [Obsidian plugin](/docs/integrations/obsidian).

Do you want to use it within WordPress? We have a [WordPress plugin](/docs/integrations/wordpress).

Do you want to use it within your Browser? We have a [Chrome extension](/docs/integrations/chrome-extension). (Firefox add-on coming soon.)

Do you want to use it within your code editor? We have documentation on how you can integrate with [Visual Studio Code and its forks](/docs/integrations/visual-studio-code), [Neovim](/docs/integrations/neovim), [Helix](/docs/integrations/helix), [Emacs](/docs/integrations/emacs), [Zed](/docs/integrations/zed) and [Sublime Text](/docs/integrations/sublime-text). If you're using a different code editor, then you can integrate directly with our language server, [`harper-ls`](/docs/integrations/language-server).

Do you want to integrate it in your web app or your JavaScript/TypeScript codebase? You can use [`harper.js`](./harperjs/introduction).

Do you want to integrate it in your Rust program or codebase? You can use [`harper-core`](https://crates.io/crates/harper-core).

## What Human Languages Do You Support?

We currently only support English and its dialects British, American, Canadian, and Australian. Other languages are on the horizon, but we want our English support to be truly amazing before we diversify.

## What Programming Languages Do You Support?

For `harper-ls` and our code editor integrations, we support a wide variety of programming languages. You can view all of them over at [the `harper-ls` documentation](/docs/integrations/language-server#Supported-Languages). We are entirely open to PRs that add support. If you just want to be able to run grammar checking on your code's comments, you can use [this PR as a model for what to do](https://github.com/Automattic/harper/pull/332).

For `harper.js` and those that use it under the hood like our Obsidian plugin, we support plaintext and/or Markdown.

## Where Did the Name Harper Come From?

See [this blog post](https://elijahpotter.dev/articles/naming_harper).

## Do I Need a GPU?

No.

Harper runs on-device, no matter what.
There are no special hardware requirements.
No GPU, no additional memory, no fuss.

## What Do I Do If My Question Isn't Here?

You can join our [Discord](https://discord.gg/invite/JBqcAaKrzQ) and ask your questions there or you can start a discussion over at [GitHub](https://github.com/Automattic/harper/discussions).

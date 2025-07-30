<div id="header" align="center">
    <img src="logo.svg" width="400px" />
    <h1>Harper</h1>
</div>

[![Harper Binaries](https://github.com/automattic/harper/actions/workflows/build_harper_binaries.yml/badge.svg)](https://github.com/automattic/harper/actions/workflows/build_harper_binaries.yml)
[![Website](https://github.com/automattic/harper/actions/workflows/build_web.yml/badge.svg)](https://github.com/automattic/harper/actions/workflows/build_web.yml)
[![Precommit](https://github.com/automattic/harper/actions/workflows/precommit.yml/badge.svg)](https://github.com/automattic/harper/actions/workflows/precommit.yml)
[![Crates.io](https://img.shields.io/crates/v/harper-ls)](https://crates.io/crates/harper-ls)
![NPM Version](https://img.shields.io/npm/v/harper.js)

Harper is an English grammar checker designed to be _just right._
I created it after years of dealing with the shortcomings of the competition.

Grammarly was too expensive and too overbearing.
Its suggestions lacked context, and were often just plain _wrong_.
Not to mention: it's a privacy nightmare.
Everything you write with Grammarly is sent to their servers.
Their privacy policy claims they don't sell the data, but that doesn't mean they don't use it to train large language models and god knows what else.
Not only that, but the round-trip-time of the network request makes revising your work all the more tedious.

LanguageTool is great, if you have gigabytes of RAM to spare and are willing to download the ~16GB n-gram dataset.
Besides the memory requirements, I found LanguageTool too slow: it would take several seconds to lint even a moderate-size document.

That's why I created Harper: it is the grammar checker that fits my needs.
Not only does it take milliseconds to lint a document, take less than 1/50th of LanguageTool's memory footprint,
but it is also completely private.

Harper is even small enough to load via [WebAssembly.](https://writewithharper.com)

## Language Support

Harper currently only supports English, but the core is extensible to support other languages, so we welcome contributions that allow for other language support.

## Performance Issues

We consider long lint times bugs.
If you encounter any significant performance issues, please create an issue on the topic.

If you find a fix to any performance issue, we are open the contribution.
Just make sure to read [our contribution guidelines first.](https://github.com/automattic/harper/blob/master/CONTRIBUTING.md)

## Links

- [Frequently Asked Questions](https://writewithharper.com/docs/faq)
- [Obsidian Documentation](https://writewithharper.com/docs/integrations/obsidian)
- [`harper-ls` Documentation](https://writewithharper.com/docs/integrations/language-server)
- Supported Editors' Documentation
  - [Visual Studio Code](https://writewithharper.com/docs/integrations/visual-studio-code)
  - [Neovim](https://writewithharper.com/docs/integrations/neovim)
  - [Helix](https://writewithharper.com/docs/integrations/helix)
  - [Emacs](https://writewithharper.com/docs/integrations/emacs)
  - [Zed](https://writewithharper.com/docs/integrations/zed)
- [`harper.js` Documentation](https://writewithharper.com/docs/harperjs/introduction)
- [Official Discord Server](https://discord.com/invite/JBqcAaKrzQ)

## Huge Thanks

This project would not be possible without the hard work from those who [contribute](https://writewithharper.com/docs/contributors/introduction).

<a href="https://github.com/automattic/harper/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=automattic/harper" />
</a>

Harper's logo was designed by [Lukas Werner](https://lukaswerner.com/).

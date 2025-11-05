---
title: Harper for Obsidian
---

Put simply, [Harper](https://writewithharper.com/) is a grammar checking plugin that doesn't violate your privacy.
Other systems, like LanguageTool, ship your writing over the internet to centralized servers, where it's used for god-knows-what.
Harper isn't like that.

Instead, Harper runs its grammar checking engine directly _inside Obsidian_.
That means your data doesn't go anywhere you don't want it to.
Your Obsidian vault should be just what you expect: locked down and private.

![A screenshot of Obsidian with Harper installed](/images/obsidian_screenshot.webp)

Unlike other offerings (like Grammarly) Harper also explicitly ignores the contents of code fences and inline code blocks.
Since it runs entirely on-device, Harper also ends up being noticeably faster than alternatives, partly because there is no network latency.

### Open Source

- Harper is fully open source, allowing transparency and community contributions to improve its functionality.
- Developers can review the codebase or contribute directly via the [GitHub repository](https://github.com/automattic/harper).

## How It Compares to Other Plugins

| Feature                | Harper                        | LanguageTool                      |
| ---------------------- | ----------------------------- | --------------------------------- |
| **Privacy**            | 100% offline                  | Requires self-hosting for privacy |
| **Real-Time Checking** | Yes                           | Yes                               |
| **Language Support**   | English (extensible) | 30+ languages                     |
| **Open Source**        | Yes                           | Partially                         |
| **Ease of Use**        | Simple setup                  | Requires API/self-hosting setup   |
| **Performance**        | Fast and lightweight          | Resource-intensive                |

## Installation Guide

1. Open Obsidian and navigate to **Settings → Community Plugins → Browse**.
2. Search for "Harper" in the plugin library.
3. Click "Install" and then "Enable."
4. Start typing in your notes—Harper will automatically highlight errors as you go!

> **Warning**
> Harper expects an up-to-date version of the Obsidian installer. If you have issues, [reinstall Obsidian](https://obsidian.md/download) or otherwise update your installer version.

## Where's all the code?

All the code for the Harper Obsidian plugin lives [in the main Harper monorepo](https://github.com/automattic/harper/tree/master/packages/obsidian-plugin).
This repository exists to satisfy the [requirements](https://docs.obsidian.md/Plugins/Releasing/Submit+your+plugin) laid out by the Obsidian team for their plugins.

## I have a problem or feature request...

Let me know if you have any problems, feature requests, or feedback of any kind by filling out an [issue on the main repository](https://github.com/automattic/harper/issues/new).

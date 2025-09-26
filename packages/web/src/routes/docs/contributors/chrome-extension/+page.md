---
title: Chrome Extension
---

Harper's Chrome extension is still in its infancy.
At a high level, there are just three components: the content script, the options page and the popup "page".

At the moment, this document is also in its infancy.
It is incomplete, and we would _really appreciate_ contributions to make it better.

![The Chrome extension's high-level architecture.](/images/chrome_extension_diagram.png)

## The Content Script

The content script has three responsibilities:

- Reading text from the user's currently open web page.
- Writing text back to the user's web page (after applying a suggestion to it).
- Rendering underlines over their text (this is the hard part).

All three of these responsibilities are handled by the `lint-framework` package.

Notably, it does not do any linting itself.
Instead, it submits requests to the background worker to do so, since instantiating a WebAssembly module on every page load is expensive.

## Popup Page

![The Chrome extension's popup page](/images/chrome_extension_popup.png)

At the moment, the popup page has just one functional button that toggles Harper on the current domain.
Again, it doesn't interact with local storage itself to do this.
Rather, it initiated requests to the background worker, which then interfaces with local storage.

## Options Page

![The Chrome extension's popup page](/images/chrome_extension_options.png)

Similar to the popup page, the options page initiates requests to the background worker to change the extensions configuration.
It has settings for:

- Changing the English dialect Harper lints for.
- Enabling/disabling individual rules

It will eventually allow users to clear ignored suggestions and configure their dictionary.

## The Background Worker

This is the location of a lot of centralized "business" logic.
It:

- Loads `harper.js` and performs linting
- Handles persistent storage and configuration of:
    - Dialect
    - Rules
    - Domain toggling

## The Firefox Extension

Despite the name of the package, the `chrome-plugin` also supports Firefox. 
To build for Firefox, just use `pnpm zip-for-firefox` or otherwise compile with the environment variable `TARGET_BROWSER=firefox`.

## Other Reading

- [Putting Harper in the Browser: Technical Details](https://elijahpotter.dev/articles/putting_harper_in_your_browser)
- [The Art of Exception](https://elijahpotter.dev/articles/the_art_of_exception)

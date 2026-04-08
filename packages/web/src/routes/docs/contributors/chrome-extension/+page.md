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

## Google Docs Support

Google Docs is a complex beast.
Used by billions around the world, it's honestly an engineering marvel.
Naturally, we want to allow Harper users to use that marvel if they so wish.
That posed a complex problem for us, which is why many of our early users may remember a (relatively) long era where we simply didn't support Google Docs.

Google Docs was difficult to support for two simple reasons:

1. It does not expose a normal editable DOM surface that Harper can lint directly.
1. The APIs that make the document readable and writable are not part of a normal web-editor integration.

Our saving grace was the [open-sourcing of Witty Works](https://github.com/witty-works/browser-extension), an inclusive language checker.
They figured out a way to access the internal document state, which we took inspiration from.
Here's how it works.

### How We Read and Write to Google Docs

At a high level, Harper uses three pieces:

1. A small bootstrap script that makes Google Docs enable the extension-facing machinery we need.
1. A main-world bridge script that can talk to Google Docs' internal annotated-text APIs.
1. A hidden mirrored target in the page that the normal lint framework can read from and render against.

That's the purpose of the `googleDocsBootstrap.js` content script.
It's short, sweet, and to the point:

@code(../../../../../../chrome-plugin/src/contentScript/googleDocsBootstrap.js)

After that, Harper injects `google-docs-bridge.js` into the page's main world.
We need that because the normal extension content script runs in an isolated world and cannot directly use the Google Docs APIs we rely on.

The bridge reads the document's logical text and selection state, watches for layout changes, and exposes enough information for the rest of Harper to work.
Harper then mirrors that state into a hidden target element inside the editor.
That hidden target gives the lint framework something stable to lint, while the visible Google Docs layout is still used to place highlights in the right location.

Suggestion application also goes back through the bridge.
In other words, the bridge is the adapter between Google Docs' internal editor model and Harper's normal browser linting flow.

## Other Reading

- [Putting Harper in the Browser: Technical Details](https://elijahpotter.dev/articles/putting_harper_in_your_browser)
- [The Art of Exception](https://elijahpotter.dev/articles/the_art_of_exception)

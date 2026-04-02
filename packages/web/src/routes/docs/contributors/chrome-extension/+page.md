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

1. It doesn't render to the DOM at all. Instead, Google Docs uses a specialized document renderer called Kix that is composed of a single `<canvas />` element.
1. The API to access the internal document state of Kix existed, but was entirely undocumented.

Our saving grace was the [open-sourcing of Witty Works](https://github.com/witty-works/browser-extension), an inclusive language checker.
They figured out a way to access the internal document state, which we took inspiration from.
Here's how it works.

### How We Read and Write to Google Docs

To get Google Docs into a readable state, we need to trick it a little bit.
Kix's API is official, but Google provides it to a limited number of "supported" vendors.
In practice, that means that if a user has one of a limited number of Chrome extensions installed, Google Docs will expose it's operable underbelly.
Despite many attempts to get on Google's whitelist, Harper is not one of these extensions.
So we have to trick it.

That's the purpose of the `googleDocsBootstrap.js` content script.
It's short, sweet, and to the point:

@code(../../../../../../chrome-plugin/src/contentScript/googleDocsBootstrap.js)

All we're doing is telling Google Docs that we're one of the supported extension, even if we technically aren't.

Once our deception is complete, Google Docs will start writing SVG nodes to the DOM, mirroring the text that's being rendered to the `<canvas />` element that we talked about earlier.

This is where our bridge comes in.
At the soonest available opportunity, Harper will inject the `google-docs-bridge.js` script into the main world of the session.
We need to do this because normal content scripts are not allowed to read or write to the APIs necessary.
This "bridge" can.

I thoroughly encourage you to read the actual source code or reach out if you have any questions.
In short, though, the bridge maintains a mirrored version of the Google Docs state, one that is readable by the normal lint framework.
In other words, it acts as middleman between the rest of the Chrome extension and Google Docs, allowing most of the existing code to be able to treat Google Docs the same way it would treat any other text editor on the web.

## Other Reading

- [Putting Harper in the Browser: Technical Details](https://elijahpotter.dev/articles/putting_harper_in_your_browser)
- [The Art of Exception](https://elijahpotter.dev/articles/the_art_of_exception)

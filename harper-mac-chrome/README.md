# HarperMac for Chrome

**HarperMac:** [Mac app](../harper-mac) · **Chrome extension** · [Safari extension](../harper-mac-safari)

A browser extension that checks your grammar and spelling in web pages. It draws the underlines and
the suggestion popup inside the page, so they line up exactly with your text, and it uses the
[HarperMac app](../harper-mac) on your Mac as the grammar engine. Nothing leaves your computer.

**The HarperMac app must be running**, or the extension has nothing to talk to.

Works in Chrome and other Chromium browsers (Brave, Edge, Arc). For Safari, this same code is
packaged by the [Safari extension](../harper-mac-safari).

## Install

This extension isn't on the Chrome Web Store, so load it by hand:

1. Start the [HarperMac app](../harper-mac).
2. In Chrome, open `chrome://extensions`.
3. Switch on **Developer mode** (top right).
4. Click **Load unpacked** and choose this folder (`harper-mac-chrome`).

Click the extension's toolbar button to see whether it's connected to the app, and to turn checking
off for the current site.

## What it checks

Any `<textarea>`, text `<input>`, or editable area (the kind used by Gmail, Notion, Slack and
similar). It skips:

- fields marked `spellcheck="false"` (code editors set this),
- password fields,
- text that looks like a URL or email address.

Click an underlined word to see the explanation and suggested replacements, or to add the word to
your dictionary or ignore the suggestion.

## How it works

| File | What it does |
|---|---|
| `content.js` | Runs in every page. Finds the field, reads its text, draws underlines, shows the popup, applies fixes |
| `background.js` | Passes messages from pages to the app, which listens on `127.0.0.1:47321` |
| `popup.html`, `popup.js` | The toolbar button's window: connection status and the per-site switch |
| `manifest.json` | Tells the browser what the extension is and what it may do |
| `make_icons.swift` | Regenerates the icons: `swift make_icons.swift` |

Pages can't call the app directly (browsers block it), so `content.js` asks `background.js`, which
makes the request. The app only accepts requests from a browser extension.

## Limits

- Editors that draw their own text on a canvas (Google Docs, for example) don't expose it, so they
  can't be checked.
- Underlines are drawn inside the page, so very unusual layouts may place them slightly off.

## Licence

Apache-2.0, the same as Harper.

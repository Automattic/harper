---
title: Using harper.js in a Chrome Extension
---

Chrome extensions are a natural place for on-device grammar checking, but they do not behave like a normal webpage when it comes to loading `harper.js`.

## Why extensions are different

- **No remote scripts.** Manifest V3 pages (popups, options pages, and service workers) use a strict default [content security policy](https://developer.chrome.com/docs/extensions/reference/manifest/content-security-policy). You generally cannot load `harper.js` from unpkg or another CDN inside the extension. Bundle it or ship a local copy of the package `dist` files with your extension.
- **Popup vs service worker.** A popup is a short-lived document. Instantiating Harper there is fine for a small demo. For a real product, load `harper.js` once in the [service worker](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers) (as Harper's own browser extension does) and message results to content scripts or the popup.
- **Workers and WASM.** `WorkerLinter` needs Worker APIs and a resolvable binary URL. In extension pages, `LocalLinter` plus the inlined binary (`harper.js/binaryInlined`) is the simplest path because the WebAssembly payload travels with your shipped files.
- **CDN links outside extensions.** On a normal website, prefer the ESM entrypoints (`dist/index.js`, `dist/binaryInlined.js`) over the older `dist/harper.js` URL. That older path does not work reliably for newer `harper.js` releases on unpkg.

## Minimal example

There is a tiny unpacked-extension example in the Harper monorepo:

[packages/harper.js/examples/chrome-extension-simple](https://github.com/Automattic/harper/tree/master/packages/harper.js/examples/chrome-extension-simple)

It opens a popup, prints the `harper.js` package version (the same release line as `harper-core`), and lints the phrase `This is an test`.

@code(../../../../../../harper.js/examples/chrome-extension-simple/popup.js)

More examples (Node, plain HTML/CDN) live under [`packages/harper.js/examples`](https://github.com/Automattic/harper/tree/master/packages/harper.js/examples).

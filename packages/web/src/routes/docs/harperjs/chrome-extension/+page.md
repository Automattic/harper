---
title: Chrome Extensions
---

`harper.js` works inside Chrome extensions, but [Manifest V3](https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3) imposes three constraints that trip people up.
This page walks through them, and [a complete working example is available in the repository](https://github.com/Automattic/harper/tree/master/packages/harper.js/examples/chrome-extension).

## Ship `harper.js` inside the extension

Extensions [may not load remotely hosted code](https://developer.chrome.com/docs/extensions/develop/migrate/improve-security#remove-remote-code), so you cannot import `harper.js` from a CDN [like a normal web page can](./CDN).
Instead, copy the package's `dist/` directory into your extension (or use a bundler) and import it with a relative path.
Importing `binaryInlined` avoids a separate fetch for the WebAssembly binary, since it ships the binary inside the JavaScript module.

## Allow WebAssembly in the content security policy

Compiling WebAssembly is blocked by the default extension content security policy.
Your `manifest.json` must opt in with [`wasm-unsafe-eval`](https://developer.chrome.com/docs/extensions/reference/manifest/content-security-policy):

@code(../../../../../../harper.js/examples/chrome-extension/src/manifest.json)

## Use `LocalLinter`

On ordinary web pages we recommend `WorkerLinter`, but it spawns its Web Worker from a blob URL, which the extension content security policy does not permit.
Inside an extension, use `LocalLinter` instead:

@code(../../../../../../harper.js/examples/chrome-extension/src/popup.js)

Note that `harper.js` does not expose a function that reports the version of the underlying engine: the `harper.js` package version shown above is released in lockstep with `harper-core`, so it identifies the engine version as well.

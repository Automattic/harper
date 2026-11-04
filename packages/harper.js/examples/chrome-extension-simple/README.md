# `chrome-extension-simple`

A minimal Manifest V3 Chrome extension that loads `harper.js` in its popup, shows the package version, and runs a one-line lint.

Chrome extensions cannot fetch remote JavaScript under the default MV3 content security policy, so this example copies a **local** `harper.js` build into `./vendor` instead of using unpkg. Prefer that pattern for extensions. The older unpkg path `dist/harper.js` is also unreliable for newer package versions; use the published ESM entrypoints when you do use a CDN on a normal webpage.

## Setup

From the Harper monorepo (with dependencies installed):

```bash
pnpm --filter harper.js build
cd packages/harper.js/examples/chrome-extension-simple
node ./prepare-extension.mjs
```

Then open `chrome://extensions`, enable Developer mode, choose **Load unpacked**, and select this directory.

Click the extension icon to open the popup.

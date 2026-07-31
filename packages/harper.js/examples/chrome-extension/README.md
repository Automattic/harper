# Chrome Extension Example

A minimal [Manifest V3](https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3) Chrome extension whose popup lints text with `harper.js` and shows the bundled Harper version.

Extensions cannot load remote code, so `build.js` copies the `harper.js` distribution into the extension directory next to the popup's own files. See [the accompanying documentation](https://writewithharper.com/docs/harperjs/chrome-extension) for a walkthrough of the extension-specific pitfalls.

To try it out:

```bash
pnpm install
pnpm build
```

Then open `chrome://extensions`, enable "Developer mode", click "Load unpacked" and select this example's `dist/` directory.

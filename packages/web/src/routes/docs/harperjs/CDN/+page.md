---
title: Using a CDN
---

You can consume Harper from the [unpkg](https://unpkg.com/) CDN using native ECMAScript module syntax which is supported by all modern browsers.

[A simple example is provided below.](./CDN/example)

@code(../../../../../../harper.js/examples/raw-web/index.html)

More runnable samples live in [`packages/harper.js/examples`](https://github.com/Automattic/harper/tree/master/packages/harper.js/examples), including a [Chrome extension](./chrome-extension) example. Prefer shipping local `dist` files inside extensions: MV3 cannot load remote scripts, and older unpkg URLs such as `dist/harper.js` do not work reliably for newer releases.

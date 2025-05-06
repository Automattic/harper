---
title: Introduction to harper.js
---

## The Mission

If you're a developer, odds are that you are using JavaScript or TypeScript on a daily basis.
Your project probably has at least a little bit of either.

Furthermore, a plurality of focused authorship happens inside either a web browser or an [Electron-based app](https://www.electronjs.org/).
Given this, we wanted to create an environment where it would be trivial to integrate fantastic grammar checking into web applications.
That's why we created `harper.js`.

Today, it serves as the foundation for our [Obsidian plugin](/docs/integrations/obsidian) and our [website](/).

## Installation

`harper.js` is an ECMAScript module designed to be easy to import into any project.
On the inside, it uses a copy of Harper's core algorithm compiled to [WebAssembly](https://webassembly.org/).

It can be imported [natively in a browser](./CDN) or through [npm](https://www.npmjs.com/package/harper.js) and [consumed in Node.js](./node).

@install-pkg(harper.js)

Notice that the `harper.js` package is currently in early access.
This means that the API is not yet stable and we are still working out the kinks.

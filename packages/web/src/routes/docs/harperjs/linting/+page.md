---
title: Linting With harper.js
---

[Linting](<https://en.wikipedia.org/wiki/Lint_(software)>) is the process of consuming, analyzing, and finding faults in text.
This is the principle task Harper tries to do.
When possible, Harper also tries to automatically generate fixes for any issues it finds.

In `harper.js`, there's just one interface you need to worry about: the `Linter`.

## Linters

The `Linter` type is relatively straightforward and has two implementations: the `LocalLinter` and the `WorkerLinter`.
Notice how every method returns a `Promise<...>`.

@code(../../../../../../harper.js/src/Linter.ts)

A `LocalLinter` will instantiate and prepare Harper's WebAssembly module asynchronously, but **in the same event loop**.
This can result in high [LCP](https://developer.mozilla.org/en-US/docs/Glossary/Largest_contentful_paint), so this implementation is only recommended in situations where the event loop will not be doing other latency-sensitive things.
In other words: `LocalLinter`s are not for the web.

A `WorkerLinter`, on the other hand, will instantiate and prepare Harper's WebAssembly module inside a [Web Worker](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API), which means it will **not** block the event loop.
This is recommended for interactive web applications.

[Visit our page about CDNs](./CDN) to see an example of a `WorkerLinter` in action, or [the page about Node.js](./node) for a `LocalLinter`.

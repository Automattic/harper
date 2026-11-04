---
title: Using Harper in Node.js
description: Run harper.js in Node.js with LocalLinter and the provided example code.
---

Harper.js can run in Node.js.
There is just one consideration: as described in [more detailed here](./linting), we cannot use the `WorkerLinter`.
That means we must use the `LocalLinter`.

Additionally, since `harper.js` is an ECMAScript module, it must be imported in a relatively recent version of Node.js.

## Example Code

The example below can be found in [the Harper monorepo.](https://github.com/Automattic/harper/tree/master/packages/harper.js/examples/commonjs-simple)

@code(../../../../../../harper.js/examples/commonjs-simple/index.js)

---
title: Set Up Your Environment
---

To use the tooling required to build and debug Harper, you'll need the following programs available in your `$PATH`.

- [`just`](https://github.com/casey/just)
- `bash`
- [`cargo`](https://www.rust-lang.org/) (we develop against the latest version of Rust)
- `yarn`
- `node`
- `grep`
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
- `zip`
- `pandoc`

We develop a set of tools, accessible via `just`, to build and debug Harper's algorithm (otherwise known as `harper-core`) and its various integrations.
The source code is in the `justfile` [at the root of the repository](https://github.com/Automattic/harper/blob/master/justfile).
To see all the tools in the toolbox, run:

```bash
just --list
```

Before making any modifications, we highly recommend that you run `just setup` to populate your build caches and download all dependencies.

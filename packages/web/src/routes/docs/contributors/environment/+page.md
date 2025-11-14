---
title: Set Up Your Environment
---

Depending on what you're wanting to work on within the Harper repo, you may not need to set up every aspect of the environment described on this page.
For example, if you only intend to work on the core grammar engine, `harper-ls`, or `harper-cli`, you only need `cargo` installed.
In that case, Harper follows all of the standard conventions for Rust projects, so any existing Rust knowledge will apply.

If you intend to work on the Chrome extension, Obsidian plugin, or certain other projects, you'll need the full environment set up.

To use _all_ the tooling we use to build and debug Harper, you'll need the following programs available in your `$PATH`.
For Nix users, we provide a [Nix development shell](#Nix-development-shell) to setup all the necessary tooling automatically.

- [`just`](https://github.com/casey/just)
- `bash`
- [`cargo`](https://www.rust-lang.org/) (we develop against the latest version of Rust)
- `pnpm`
- `node`
- `grep`
- [`wasm-pack`](https://drager.github.io/wasm-pack/installer/)
- `zip`
- [`cargo-hack`](https://github.com/taiki-e/cargo-hack?tab=readme-ov-file#installation)

To run integration tests, you may also need `libnss3` and/or `libasound3`.
These are installable in Ubuntu using `apt-get`.

```bash
sudo apt-get install libnss3
sudo apt-get install libasound2
```

We develop a set of tools, accessible via `just`, to build and debug Harper's algorithm (otherwise known as `harper-core`) and its various integrations.
The source code is in the `justfile` [at the root of the repository](https://github.com/Automattic/harper/blob/master/justfile).
To see all the tools in the toolbox, run:

```bash
just --list
```

> Please note that `just build-web` _only_ builds the website for production, while `just dev-web` also spins up a development server.

Before making any modifications, we highly recommend that you run `just setup` to populate your build caches and download all dependencies.
If you see a Visual Studio code window pop open, don't worry! That's just a part of our integration tests.

## Nix development shell

If you use [nix-direnv](https://github.com/nix-community/nix-direnv), the shell will be loaded automatically when you change into the project directory.

Otherwise, run:

```bash
nix develop
```

This will start a bash shell that provides the build environment with everything you need to start contributing!

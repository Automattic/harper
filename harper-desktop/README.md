# Harper Desktop

![A screenshot of the first version of Harper Desktop](./screenshot.png)

> __NOTICE__: Documentation for Harper Desktop is incomplete. It will be updated on a "when we have time" basis. 

For context, see these posts:

- [Build A Harper Desktop App](https://elijahpotter.dev/articles/building-a-harper-desktop-app)

Right now, Harper Desktop does little more than serve as an offline editor for Markdown with the Harper grammar checker baked in.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Building

Most actions needed to work on this repository are available via [`just`](https://github.com/casey/just).
For the time being, this project has [the same prerequisites as the Harper monorepo](https://writewithharper.com/docs/contributors/environment).
We will get to why in a second.

I have not fully decided whether Harper Desktop will become a part of the Harper monorepo. 
Until I have, we will be developing it in a private repository.
To access the bits of Harper that aren't packaged in public places, the `justfile` contains a command to pull down a copy of the Harper monorepo and build the components that are relevant to Harper Desktop.

To build those dependencies, run the following command:

```bash
just pull-dep-source build-harper-deps
```

From there, you can launch a development version of Harper Desktop (with live reload and all of those goodies) using:

```bash
just dev
```

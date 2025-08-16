---
title: Contributor's Guide to the Obsidian Plugin
---

This page will outline the most important bits of information needed to work on the Harper Obsidian plugin.
In addition to that outlined in [environment set up page](./environment), you'll also need a working [Obsidian installation](https://obsidian.md/) and vault prepared.

All the code for the Obsidian plugin lies in the `packages/obsidian-plugin` directory of our [monorepo](https://github.com/automattic/harper).

## Obsidian's Quirks

Obsidian, in the interest of relieving their development team of any overhead, imposes some restrictions on our plugins that make the build and deploy process unusual.

### Plugins Must Be a Single File

All the executable code for an Obsidian plugin must be contained within a single file.
That includes both JavaScript and—in our case—WebAssembly.

This is why the only artifact of the Harper Obsidian plugin build process is a single, [heavily minimized](https://www.cloudflare.com/learning/performance/why-minify-javascript-code/) `main.js` file.

### Plugins Are Loaded from a GitHub Repository

All Obsidian plugins are downloaded and installed from the latest release in a dedicated GitHub repository.
Since Harper uses a monorepo, our [dedicated GitHub repository](https://github.com/Automattic/harper-obsidian-plugin) is just a skeleton, containing barely more than a plugin manifest and `README.md`.

PRs for the Obsidian plugin should be __submitted to the monorepo__.
When a release is necessary, Harper maintainers will increment the version in the [skeletal repository](https://github.com/Automattic/harper-obsidian-plugin) and create a release.

## Developing the Harper Obsidian Plugin

Obsidian loads its plugins from the dedicated plugin directory inside your vault:

```
<vault dir>/.obsidian/plugins/<plugin name>
```

For Harper, this looks exactly as you'd expect:

```
<vault dir>/.obsidian/plugins/harper
```

The workflow for quickly iterating on the plugin looks something like this:

1. Compile and start watching for changes with `just setup && cd packages/obsidian-plugin && pnpm dev`.
   This will continously rebuild `main.js` anytime you make a change to a file in the `packages/obsidian-plugin` directory.
2. Copy the `manifest.json` from the skeletal repo to the dedicated plugin directory shown above.
3. Create a symbolic link to the `main.js` produced by step 1 in the dedicated plugin directory above.
4. Anytime you make a change, run the `Reload app without saving` command inside of Obsidian.

If you've done everything right, the dedicated plugin directory should contain three files:

1. `main.js`
2. `manifest.json`
3. `data.json` 

`data.json` is a configuration file created and modified by the Harper plugin to persist settings, the user's dictionary and other miscellaneous things.

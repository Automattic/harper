---
title: What Is Harper?
---

Harper is a grammar checker designed to run anywhere there is text (so really, anywhere).
Most Harper users are catching their mistakes in [Neovim](./integrations/neovim), [Obsidian](./integrations/obsidian), or [Visual Studio Code](./integrations/visual-studio-code).

<script>
    import Editor from "$lib/components/Editor.svelte"
</script>

<div class="h-96">
    <Editor content={`You can try out a editor that uses\nHarper under the hood here.\n\nIt is rnning in your browser right now. \n\nNo server required!`}/>
</div>

## How Does It Work?

Harper takes advantage of decades of natural language research to analyze exactly how your words come together.
If something is off, Harper lets you know.

In a way, Harper is an error-tolerant parser for English.

## Versioning Policy

Harper uses [semantic versioning](https://semver.org/).

All components and integrations of Harper stay in version sync, including but not limited to:

- `harper.js`
- `harper-core`
- `harper-comments`
- `harper-ls`
- The Obsidian Plugin
- The VS Code Plugin
- The Chrome Extension

That means that a change in `harper.js` can cause a release of the Obsidian plugin with a version bump, even if nothing has directly changed in the Obsidian plugin.
We do this because we view Harper not as a disparate set of integrations, but as a holistic system accessible in a wide variety of places.

For the time being, we only actively develop and maintain the latest version of Harper (seen in the `master` branch on GitHub). 
If long-term support for older versions is desired, please let us know and we will do our best to accommodate you.

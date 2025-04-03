---
title: What Is Harper?
---

Harper is a grammar checker designed to run anywhere there is text (so really, anywhere).
Most Harper users are catching their mistakes in Neovim, [Obsidian](./integrations/obsidian), or Visual Studio Code.

<script>
    import Editor from "$lib/Editor.svelte"
</script>

<div class="h-96">
    <Editor content={`You can try out a editor that uses\nHarper under the hood here.\n\nIt is rnning in your browser right now. \n\nNo server required!`}/>
</div>

## How Does It Work?

Harper takes advantage of decades of natural language research to analyze exactly how your words come together.
If something is off, Harper lets you know.

In a way, Harper is an error-tolerant parser for English.

## How Can I Start Using Harper?

We have a number of integrations available for a number of use-cases, ranging from [Obsidian for note-taking](./integrations/obsidian) to [Neovim](./integrations/neovim) or [Visual Studio Code](./integrations/visual-studio-code) for programming.

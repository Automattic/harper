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

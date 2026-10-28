<script lang="ts">
import type { Linter } from 'harper.js';
import { onMount } from 'svelte';
import Editor from '../Editor.svelte';

export let content = 'Try teh Harper editor. It checks your writing as you type.';
/** Wrap the editor in a `.dark` ancestor to show its dark palette. */
export let dark = false;
let linter: Linter | undefined;
let error = '';

onMount(() => {
	let active = true;
	/** Load the browser WASM linter before mounting the editor. */
	async function setup() {
		try {
			const [{ LocalLinter }, { slimBinary }] = await Promise.all([
				import('harper.js'),
				import('harper.js/slimBinary'),
			]);
			const instance = new LocalLinter({ binary: slimBinary });
			await instance.setup();
			if (active) linter = instance;
		} catch (cause) {
			if (active) error = String(cause);
		}
	}
	void setup();
	return () => {
		active = false;
	};
});
</script>

<div class:dark style="width: min(900px, 90vw); height: 500px;">
	{#if linter}
		<Editor {content} {linter} />
	{:else if error}
		<p>Could not start Harper: {error}</p>
	{:else}
		<p>Loading Harper…</p>
	{/if}
</div>

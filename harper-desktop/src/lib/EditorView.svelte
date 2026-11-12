<script lang="ts">
import { listen } from '@tauri-apps/api/event';
import type { Linter } from 'harper.js';
import { Editor } from 'harper-editor';
import { onMount, tick } from 'svelte';
import { Client } from './client';
import 'harper-editor/style.css';

let linter: Linter | null = null;
let error = '';

onMount(() => {
	let active = true;
	let pending = Promise.resolve();
	// Serialize refreshes so an older pack snapshot cannot replace a newer one.
	const refresh = () => {
		pending = pending
			.then(async () => {
				if (!active) return;
				const next = await Client.createEditorLinter();
				if (!active) {
					await next.dispose();
					return;
				}
				const previous = linter;
				if (previous) await next.importIgnoredLints(await previous.exportIgnoredLints());
				linter = next;
				error = '';
				await tick();
				await previous?.dispose();
			})
			.catch((cause) => {
				if (active) error = String(cause);
			});
	};
	const unlisten = listen('weirpacks-changed', refresh);
	void unlisten.then(refresh).catch((cause) => {
		if (active) error = String(cause);
	});
	return () => {
		active = false;
		void unlisten.then((stop) => stop()).catch(console.error);
		void pending.then(() => linter?.dispose());
	};
});
</script>

<div class="h-screen w-screen">
    {#if error}<p role="alert">Failed to load editor settings: {error}</p>{/if}
    {#if linter}
        <Editor {linter} />
    {/if}
</div>

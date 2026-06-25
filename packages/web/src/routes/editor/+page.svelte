<script lang="ts">
/// This page exists to be embedded via an `iframe`.

import type { Linter } from 'harper.js';
import { Editor } from 'harper-editor';
import { onMount } from 'svelte';
import { page } from '$app/stores';
import Isolate from '$lib/components/Isolate.svelte';
import { createEditorLinter } from '$lib/createEditorLinter';

let content = $page.url.searchParams.get('initialText') ?? '';
let linter: Linter | null = null;

onMount(() => {
	(async () => {
		linter = await createEditorLinter();
	})();
});
</script>

<Isolate>
	{#if linter}
		<Editor {content} {linter}></Editor>
	{/if}
</Isolate>

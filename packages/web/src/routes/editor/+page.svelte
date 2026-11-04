<script module lang="ts">
export const frontmatter = {
	title: 'Harper Web Editor',
	description:
		'Check English grammar and spelling in the Harper web editor and review suggested corrections.',
};
</script>

<PageMetadata metadata={frontmatter} />
<h1 class="mt-0 mb-5 text-[32px]! leading-[1.3]! font-bold! tracking-[-0.02em]! text-(--docs-heading)!">{frontmatter.title}</h1>

<script lang="ts">
import PageMetadata from '$lib/PageMetadata.svelte';
import EditPage from '$lib/docs/EditPage.svelte';
/// This page exists to be embedded via an `iframe`.

import { Isolate } from 'components';
import type { Linter } from 'harper.js';
import { onMount } from 'svelte';
import { page } from '$app/stores';
import { createEditorLinter } from '$lib/createEditorLinter';

let content = $page.url.searchParams.get('initialText') ?? '';
let linter: Linter | null = null;
let Editor: typeof import('harper-editor').Editor | null = null;

onMount(() => {
	void (async () => {
		const [editorModule, instance] = await Promise.all([
			import('harper-editor'),
			createEditorLinter(),
		]);
		Editor = editorModule.Editor;
		linter = instance;
	})();
});
</script>

<Isolate>
	{#if Editor && linter}
		<svelte:component this={Editor} {content} {linter} />
	{:else}
		<p role="status">Loading Harper's grammar engine...</p>
	{/if}
</Isolate>
<EditPage sourcePath="editor/+page.svelte" />

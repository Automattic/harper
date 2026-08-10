<script lang="ts">
import type { Linter } from 'harper.js';
import { Editor } from 'harper-editor';
import { onMount } from 'svelte';
import 'harper-editor/style.css';

let linter: Linter | null = null;

onMount(() => {
	void createLinter();
});

async function createLinter() {
	const [{ WorkerLinter }, { slimBinaryInlined }] = await Promise.all([
		import('harper.js'),
		import('harper.js/slimBinaryInlined'),
	]);

	const nextLinter = new WorkerLinter({ binary: slimBinaryInlined });
	await nextLinter.setup();
	linter = nextLinter;
}
</script>

<div class="h-screen w-screen">
	{#if linter}
		<Editor {linter} />
	{/if}
</div>

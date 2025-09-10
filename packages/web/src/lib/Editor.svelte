<script lang="ts">
import LintCard from '$lib/LintCard.svelte';
import { Card } from 'flowbite-svelte';
import { type WorkerLinter } from 'harper.js';
import {
	applySuggestion,
	LintFramework,
	type UnpackedLint,
	type UnpackedSuggestion,
	unpackLint,
} from 'lint-framework';
import demo from '../../../../demo.md?raw';

export let content = demo.trim();

let editor: HTMLTextAreaElement | null;
let linter: WorkerLinter;

// Live list of lints from the framework's lint callback
let lints: UnpackedLint[] = [];
let openIndex: number | null = null;

let lfw = new LintFramework(async (text) => {
	// Guard until the linter is ready
	if (!linter) return [];

	const raw = await linter.lint(text);
	// The framework expects "unpacked" lints with plain fields
	const unpacked = await Promise.all(
		raw.map((lint) => unpackLint(window.location.hostname, lint, linter)),
	);

	lints = unpacked;

	return unpacked;
}, {});

(async () => {
	let { WorkerLinter, binary } = await import('harper.js');
	linter = new WorkerLinter({ binary });

	await linter.setup();
})();

$: if (editor != null) {
	lfw.addTarget(editor);
}

function applySug(lint: UnpackedLint, s: UnpackedSuggestion) {
	content = applySuggestion(content, lint.span, s);
	// Trigger re-lint and rerender after programmatic change
	lfw.update();
}

function createSnippetFor(lint: UnpackedLint) {
	const CONTEXT = 60;
	const start = Math.max(0, lint.span.start - CONTEXT);
	const end = Math.min(content.length, lint.span.end + CONTEXT);

	let prefix = content.slice(start, lint.span.start);
	let suffix = content.slice(lint.span.end, end);

	// Collapse whitespace/newlines for a compact blurb
	const collapse = (s: string) => s.replace(/\s+/g, ' ').trim();
	prefix = collapse(prefix);
	const problem = collapse(lint.problem_text);
	suffix = collapse(suffix);

	return {
		prefix,
		problem,
		suffix,
		prefixEllipsis: start > 0,
		suffixEllipsis: end < content.length,
	};
}
</script>

<div class="flex flex-row h-full max-w-full">
	<Card class="flex-1 h-full p-5 z-10 max-w-full text-lg mr-5">
		<textarea
			bind:this={editor}
			class="w-full m-0 rounded-none p-0 z-0 bg-transparent h-full border-none text-lg resize-none focus:border-0"
			bind:value={content}
		></textarea>
	</Card>

	<Card class="hidden md:flex md:flex-col md:w-1/3 h-full p-5 z-10">
		<div class="text-base font-semibold mb-3">Problems</div>
		<div class="flex-1 overflow-y-auto pr-1">
			{#if lints.length === 0}
				<p class="text-sm text-gray-500">No lints yet.</p>
			{:else}
                <div class="space-y-3">
                    {#each lints as lint, i}
                        <LintCard
                            {lint}
                            snippet={createSnippetFor(lint)}
                            open={openIndex === i}
                            onToggle={() => (openIndex = openIndex === i ? null : i)}
                            onApply={(s) => applySug(lint, s)}
                        />
                    {/each}
                </div>
            {/if}
        </div>
    </Card>
</div>

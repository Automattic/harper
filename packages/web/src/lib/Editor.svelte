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
// Track which lint cards are open by index
let openSet: Set<number> = new Set();

let lfw = new LintFramework(
	async (text) => {
		if (!linter) return [];

		const raw = await linter.lint(text);
		// The framework expects "unpacked" lints with plain fields
		const unpacked = await Promise.all(raw.map((lint) => unpackLint(text, lint, linter)));

		lints = unpacked;

		return unpacked;
	},
	{
		ignoreLint: async (hash: string) => {
			if (!linter) return;
			try {
				await linter.ignoreLintHash(BigInt(hash));
				console.log(`Ignored ${hash}`);
				// Re-run linting to hide ignored lint immediately
				lfw.update();
			} catch (e) {
				console.error('Failed to ignore lint', e);
			}
		},
	},
);

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

function jumpTo(lint: UnpackedLint) {
	if (!editor) return;
	const start = lint.span.start;
	const end = lint.span.end;
	// Focus and select; most browsers will scroll selection into view on focus
	editor.focus();
	editor.setSelectionRange(start, end);
	// As a fallback, nudge scroll to selection if needed
	try {
		const approxLineHeight = 20;
		const beforeText = content.slice(0, start);
		const line = (beforeText.match(/\n/g)?.length ?? 0) + 1;
		const targetTop = Math.max(0, (line - 3) * approxLineHeight);
		(editor as HTMLTextAreaElement).scrollTop = targetTop;
	} catch {}
}

function toggleCard(i: number) {
	const wasOpen = openSet.has(i);
	if (wasOpen) {
		const ns = new Set(openSet);
		ns.delete(i);
		openSet = ns;
	} else {
		const ns = new Set(openSet);
		ns.add(i);
		openSet = ns;
	}
}

$: allOpen = lints.length > 0 && openSet.size === lints.length;

function toggleAll() {
	if (allOpen) {
		openSet = new Set();
	} else {
		openSet = new Set(lints.map((_, i) => i));
	}
}

async function ignoreAll() {
	if (!linter || lints.length === 0) return;
	try {
		const hashes = Array.from(new Set(lints.map((l) => l.context_hash)));
		await Promise.all(hashes.map((h) => linter.ignoreLintHash(BigInt(h))));
		// Refresh to hide ignored lints immediately
		lfw.update();
	} catch (e) {
		console.error('Failed to ignore all lints', e);
	}
}

// Keep openSet in range if lint list changes
$: if (openSet.size > 0) {
	const max = lints.length;
	const next = new Set<number>();
	for (const idx of openSet) {
		if (idx >= 0 && idx < max) next.add(idx);
	}
	if (next.size !== openSet.size) openSet = next;
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
		<div class="flex items-center justify-between mb-3">
			<div class="text-base font-semibold">Problems</div>
			<div class="flex items-center gap-2">
				<button
					class="text-xs px-2 py-1 rounded border border-gray-300 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-[#0b0f14]"
					on:click={toggleAll}
					aria-label={allOpen ? 'Collapse all lint cards' : 'Open all lint cards'}
				>
					{allOpen ? 'Collapse all' : 'Open all'}
				</button>
				<button
					class="text-xs px-2 py-1 rounded border border-gray-300 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-[#0b0f14]"
					on:click={ignoreAll}
					disabled={lints.length === 0}
					aria-label="Ignore all current lints"
				>
					Ignore all
				</button>
			</div>
		</div>
		<div class="flex-1 overflow-y-auto pr-1">
			{#if lints.length === 0}
				<p class="text-sm text-gray-500">No lints yet.</p>
			{:else}
                <div class="space-y-3">
                    {#each lints as lint, i}
                        <LintCard
                            {lint}
                            snippet={createSnippetFor(lint)}
                            open={openSet.has(i)}
                            onToggleOpen={() => toggleCard(i)}
                            focusError={() => jumpTo(lint)}
                            onApply={(s) => applySug(lint, s)}
                        />
                    {/each}
                </div>
            {/if}
        </div>
    </Card>
</div>

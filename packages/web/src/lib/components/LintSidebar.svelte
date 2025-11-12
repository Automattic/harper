<script lang="ts">
import { Card } from 'flowbite-svelte';
import { type IgnorableLintBox, type LintBox, type UnpackedLint } from 'lint-framework';
import LintCard from '$lib/components/LintCard.svelte';

export let lintBoxes: IgnorableLintBox[] = [];
export let focusLint: (lintBox: IgnorableLintBox) => void = () => {};

async function ignoreAll() {
	await Promise.all(lintBoxes.map((b) => (b.ignoreLint ? b.ignoreLint() : Promise.resolve())));
}

let openSet: Set<number> = new Set();

$: allOpen = lintBoxes.length > 0 && openSet.size === lintBoxes.length;

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

function toggleAll() {
	if (allOpen) {
		openSet = new Set();
	} else {
		openSet = new Set(lintBoxes.map((_, i) => i));
	}
}

function collapse(contents: string) {
	return contents.replace(/\s+/g, ' ').trim();
}

function createSnippetFor(lintBox: LintBox) {
	let lint = lintBox.lint;
	let content = lintBox.source.textContent;

	const CONTEXT = 60;
	const start = Math.max(0, lint.span.start - CONTEXT);
	const end = Math.min(content.length, lint.span.end + CONTEXT);

	let prefix = content.slice(start, lint.span.start);
	let suffix = content.slice(lint.span.end, end);

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

$: if (openSet.size > 0) {
	const max = lintBoxes.length;
	const next = new Set<number>();
	for (const idx of openSet) {
		if (idx >= 0 && idx < max) next.add(idx);
	}
	if (next.size !== openSet.size) openSet = next;
}
</script>

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
				disabled={lintBoxes.length === 0}
				aria-label="Ignore all current lints"
			>
				Ignore all
			</button>
		</div>
	</div>
	<div class="flex-1 overflow-y-auto pr-1">
		{#if lintBoxes.length === 0}
			<p class="text-sm text-gray-500">No lints yet.</p>
		{:else}
			<div class="space-y-3">
				{#each lintBoxes as lintBox, i}
					<LintCard
						lint={lintBox.lint}
						snippet={createSnippetFor(lintBox)}
						open={openSet.has(i)}
						onToggleOpen={() => toggleCard(i)}
						focusError={() => focusLint(lintBox)}
						onApply={(s) => lintBox.applySuggestion(s)}
					/>
				{/each}
			</div>
		{/if}
	</div>
</Card>

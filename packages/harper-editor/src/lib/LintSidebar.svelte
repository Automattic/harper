<script lang="ts">
import {
	createEmptyCategoryCounts,
	displayCategoryFor,
	LINT_CATEGORY_ENTRIES,
	lintBoxId,
} from './editorDisplay.js';
import LintCard from './LintCard.svelte';
import type { IgnorableLintBox, LintBox } from './types.js';

export let lintBoxes: IgnorableLintBox[] = [];
export let activeLintId: string | null = null;
export let focusLint: (lintBox: IgnorableLintBox) => void = () => {};
export let onActivate: (lintBox: IgnorableLintBox | null) => void = () => {};
export let onApplied: () => void = () => {};
export let onIgnored: () => void = () => {};

let openSet: Set<string> = new Set();
let previousSignature = '';
let showCategoryCounts = true;

const headerButtonClass =
	'inline-flex h-[24px] items-center justify-center whitespace-nowrap rounded-md border-[0.5px] border-stone-300 bg-linear-to-b from-white to-stone-50 px-2.5 text-[12.5px] font-medium text-stone-950 shadow-sm shadow-stone-950/5 disabled:opacity-50';

$: allOpen = lintBoxes.length > 0 && openSet.size === lintBoxes.length;
$: counts = lintBoxes.reduce((acc, lintBox) => {
	acc[displayCategoryFor(lintBox.lint.lint_kind)] += 1;
	return acc;
}, createEmptyCategoryCounts());
$: visibleCategoryEntries = LINT_CATEGORY_ENTRIES.filter(([key]) => counts[key] > 0);
$: signature = lintBoxes.map(lintBoxId).join('|');
$: if (signature !== previousSignature) {
	previousSignature = signature;
	const availableIds = new Set(lintBoxes.map(lintBoxId));
	const next = new Set([...openSet].filter((id) => availableIds.has(id)));

	if (next.size === 0 && lintBoxes.length > 0) {
		next.add(lintBoxId(lintBoxes[0]));
	}

	openSet = next;
}

async function ignoreAll() {
	await Promise.all(lintBoxes.map((b) => (b.ignoreLint ? b.ignoreLint() : Promise.resolve())));
	openSet = new Set();
	onIgnored();
}

function toggleCard(id: string) {
	const next = new Set(openSet);
	if (next.has(id)) {
		next.delete(id);
	} else {
		next.add(id);
	}
	openSet = next;
}

function toggleAll() {
	if (allOpen) {
		openSet = new Set();
		return;
	}

	openSet = new Set(lintBoxes.map(lintBoxId));
}

function collapse(contents: string) {
	return contents.replace(/\s+/g, ' ').trim();
}

function createSnippetFor(lintBox: LintBox) {
	let lint = lintBox.lint;
	let content = lint.source || lintBox.source.textContent || '';

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
</script>

<aside
	class="flex min-h-0 w-[320px] flex-[0_0_320px] flex-col border-l-[0.5px] border-[rgba(28,26,22,0.14)] bg-[#f4f0e7] [font-family:'Inter',-apple-system,BlinkMacSystemFont,'SF_Pro_Text','Helvetica_Neue',sans-serif] @max-[760px]:w-full @max-[760px]:flex-[0_0_42%] @max-[760px]:border-t-[0.5px] @max-[760px]:border-l-0"
	aria-label="Problems"
>
	<header class="flex items-center gap-1.5 px-3.5 pt-2.5 pb-2">
		<h2
			class="!m-0 flex min-w-0 flex-1 items-center !p-0 !text-[15px] !leading-none !font-bold text-stone-950 ![font-family:inherit]"
		>
			<button
				type="button"
				class="!m-0 inline-flex min-w-0 items-center gap-1.5 border-0 bg-transparent !p-0 text-left !text-[15px] !leading-none font-[inherit] text-inherit"
				aria-expanded={showCategoryCounts}
				on:click={() => (showCategoryCounts = !showCategoryCounts)}
			>
				Problems
					<span
						class="inline-flex h-[18px] min-w-5 items-center justify-center rounded-full bg-amber-700 px-1.5 text-[11px] font-semibold text-white tabular-nums"
					>
						{lintBoxes.length}
					</span>
					<span
						class={`inline-flex shrink-0 text-stone-500 transition-transform duration-150 ${
							showCategoryCounts ? 'rotate-180' : ''
						}`}
					>
					<svg
						viewBox="0 0 16 16"
						aria-hidden="true"
						class="h-3.5 w-3.5 fill-none stroke-current stroke-[1.6] [stroke-linecap:round] [stroke-linejoin:round]"
					>
						<path d="M4 6 8 10 12 6" />
					</svg>
				</span>
			</button>
		</h2>
		<div class="flex shrink-0 items-center gap-1.5">
			<button
				type="button"
				class={headerButtonClass}
				on:click={toggleAll}
				disabled={lintBoxes.length === 0}
			>
				{allOpen ? 'Collapse all' : 'Open all'}
			</button>
			<button
				type="button"
				class={headerButtonClass}
				on:click={ignoreAll}
				disabled={lintBoxes.length === 0}>Ignore all</button
			>
		</div>
	</header>

		{#if showCategoryCounts && visibleCategoryEntries.length > 0}
			<div
				class="grid grid-cols-3 gap-x-2 gap-y-[7px] overflow-hidden border-b-[0.5px] border-[rgba(28,26,22,0.09)] px-[18px] pb-3"
				aria-label="Problem categories"
			>
				{#each visibleCategoryEntries as [key, category]}
					<div
						class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1 whitespace-nowrap text-[10px] font-medium text-stone-500"
					>
						<span
							class={`inline-flex h-[9px] w-[9px] shrink-0 items-center justify-center rounded-full ${category.haloClass}`}
						>
							<span class={`h-[7px] w-[7px] rounded-full ${category.dotClass}`}></span>
						</span>
						<span class="overflow-hidden text-ellipsis">{category.label}</span>
						<strong class="font-medium text-stone-400 tabular-nums">{counts[key]}</strong>
					</div>
				{/each}
			</div>
	{/if}

		<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-auto px-3.5 pt-3.5 pb-6" data-problems-scroller>
			{#if lintBoxes.length === 0}
				<div
					class="m-auto max-w-[220px] px-3 py-7 text-center text-[12.5px] leading-[1.55] text-stone-500"
				>
					<div
						aria-hidden="true"
						class="mb-2.5 inline-flex h-8 w-8 items-center justify-center rounded-lg bg-linear-to-b from-emerald-400 to-emerald-600 text-white shadow-sm shadow-emerald-900/20"
					>
					<svg
						viewBox="0 0 16 16"
						class="h-4 w-4 fill-none stroke-current stroke-[1.6] [stroke-linecap:round] [stroke-linejoin:round]"
					>
						<path d="M3.5 8.5 6.5 11.5 12.5 5" />
					</svg>
				</div>
				<strong class="mb-0.5 block font-semibold text-stone-950">All clear</strong>
				<p class="m-0">Harper has no suggestions for this document.</p>
			</div>
		{:else}
			{#each lintBoxes as lintBox}
				{@const id = lintBoxId(lintBox)}
				<LintCard
					lint={lintBox.lint}
					snippet={createSnippetFor(lintBox)}
					open={openSet.has(id)}
					active={activeLintId === id}
					onToggleOpen={() => toggleCard(id)}
					focusError={() => focusLint(lintBox)}
					onActivate={() => onActivate(lintBox)}
					onApply={(suggestion) => {
						lintBox.applySuggestion(suggestion);
						onApplied();
					}}
					onIgnore={async () => {
						await lintBox.ignoreLint?.();
						onIgnored();
					}}
				/>
			{/each}
		{/if}
	</div>
</aside>

<style>
	aside :global(code) {
		font-family: inherit;
		font-size: inherit;
	}
</style>

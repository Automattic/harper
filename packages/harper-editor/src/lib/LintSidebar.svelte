<script lang="ts">
import { cubicOut } from 'svelte/easing';
import { fade, fly } from 'svelte/transition';
import { createEmptyLintKindCounts, LINT_KIND_STYLE_ENTRIES } from './editorDisplay.js';
import LintCard from './LintCard.svelte';
import type { IgnorableLintBox, LintBox } from './types.js';

export let lintBoxes: IgnorableLintBox[] = [];
export let activeLintId: string | null = null;
export let focusLint: (lintBox: IgnorableLintBox) => void = () => {};
export let onActivate: (lintBox: IgnorableLintBox | null) => void = () => {};
export let onApplied: () => void = () => {};
export let onIgnored: () => void = () => {};
export let onIgnoreAll: (() => void | Promise<void>) | null = null;
export let onHideSidebar: () => void = () => {};

let openSet: Set<string> = new Set();
let menuRoot: HTMLDivElement | null = null;
let previousSignature = '';
let showLintKindCounts = true;
let menuOpen = false;
let showIgnoreConfirm = false;

const iconButtonClass =
	'inline-flex h-7 w-7 items-center justify-center rounded-md border-0 bg-transparent text-stone-600 shadow-none transition-colors duration-150 hover:text-stone-950 disabled:opacity-50';
const menuItemClass =
	'm-0 flex h-8 w-full items-center border-0 bg-transparent px-3 text-left text-[13px] font-medium text-stone-700 shadow-none hover:bg-stone-100 disabled:text-stone-300 disabled:hover:bg-transparent';
const dangerMenuItemClass =
	'm-0 flex h-8 w-full items-center border-0 bg-transparent px-3 text-left text-[13px] font-medium text-red-700 shadow-none hover:bg-red-50 disabled:text-stone-300 disabled:hover:bg-transparent';

$: allOpen = lintBoxes.length > 0 && openSet.size === lintBoxes.length;
$: problemCountLabel = `${lintBoxes.length} ${lintBoxes.length === 1 ? 'problem' : 'problems'}`;
$: counts = lintBoxes.reduce((acc, lintBox) => {
	acc[lintBox.lint.lint_kind] += 1;
	return acc;
}, createEmptyLintKindCounts());
$: visibleLintKindEntries = LINT_KIND_STYLE_ENTRIES.filter(([key]) => counts[key] > 0);
$: signature = lintBoxes.map((lintBox) => lintBox.lint.context_hash).join('|');
$: if (signature !== previousSignature) {
	previousSignature = signature;
	const availableIds = new Set(lintBoxes.map((lintBox) => lintBox.lint.context_hash));
	const next = new Set([...openSet].filter((id) => availableIds.has(id)));

	if (next.size === 0 && lintBoxes.length > 0) {
		next.add(lintBoxes[0].lint.context_hash);
	}

	openSet = next;
}

async function ignoreAll() {
	if (onIgnoreAll != null) {
		await onIgnoreAll();
	} else {
		const boxesToIgnore = [...lintBoxes];
		for (const lintBox of boxesToIgnore) {
			await lintBox.ignoreLint?.();
		}
	}

	openSet = new Set();
	onIgnored();
}

function hideSidebar() {
	menuOpen = false;
	showIgnoreConfirm = false;
	onHideSidebar();
}

function openAllCards() {
	openSet = new Set(lintBoxes.map((lintBox) => lintBox.lint.context_hash));
	menuOpen = false;
}

function closeAllCards() {
	openSet = new Set();
	menuOpen = false;
}

function requestIgnoreAll() {
	menuOpen = false;
	showIgnoreConfirm = true;
}

async function confirmIgnoreAll() {
	showIgnoreConfirm = false;
	await ignoreAll();
}

function cancelIgnoreAll() {
	showIgnoreConfirm = false;
}

function handleIgnoreBackdropClick(event: MouseEvent) {
	if (event.target === event.currentTarget) {
		cancelIgnoreAll();
	}
}

function handleWindowClick(event: MouseEvent) {
	if (!menuOpen || menuRoot == null || event.target == null) {
		return;
	}

	if (!menuRoot.contains(event.target as Node)) {
		menuOpen = false;
	}
}

function handleWindowKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape') {
		return;
	}

	if (showIgnoreConfirm) {
		event.preventDefault();
		showIgnoreConfirm = false;
		return;
	}

	if (menuOpen) {
		event.preventDefault();
		menuOpen = false;
	}
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

<svelte:window on:click={handleWindowClick} on:keydown={handleWindowKeydown} />

<aside
	class="flex min-h-0 w-[320px] flex-[0_0_320px] overflow-hidden border-l-[0.5px] border-[rgba(28,26,22,0.14)] bg-[#f4f0e7] @max-[760px]:w-full @max-[760px]:flex-[0_0_42%] @max-[760px]:border-t-[0.5px] @max-[760px]:border-l-0"
	aria-label="Problems"
	transition:fly={{ x: 320, duration: 250, easing: cubicOut }}
>
	<div class="flex min-h-0 flex-1 flex-col" transition:fade={{ duration: 250 }}>
		<header class="flex items-center gap-1.5 px-3.5 pt-2.5 pb-2">
			<h2
				class="m-0! flex min-w-0 flex-1 items-center p-0! text-[15px]! leading-none! font-bold! text-stone-950 font-[inherit]!"
			>
				<button
					type="button"
					class="m-0! inline-flex min-w-0 items-center gap-1.5 border-0 bg-transparent p-0! text-left text-[15px]! leading-none! font-[inherit] text-inherit"
					aria-expanded={showLintKindCounts}
					on:click={() => (showLintKindCounts = !showLintKindCounts)}
				>
					Problems
					<span
						class="inline-flex h-[18px] min-w-5 items-center justify-center rounded-full bg-amber-700 px-1.5 text-[11px] font-semibold text-white tabular-nums"
					>
						{lintBoxes.length}
					</span>
					<span
						class={`inline-flex shrink-0 text-stone-500 transition-transform duration-150 ${
							showLintKindCounts ? 'rotate-180' : ''
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

			<div bind:this={menuRoot} class="relative flex shrink-0 items-center gap-1.5">
				<button
					type="button"
					class={iconButtonClass}
					aria-label="Hide problems sidebar"
					title="Hide problems sidebar"
					on:click={hideSidebar}
				>
					<svg
						viewBox="0 0 20 20"
						aria-hidden="true"
						class="h-[18px] w-[18px] fill-none stroke-current stroke-[1.5] [stroke-linecap:round] [stroke-linejoin:round]"
					>
						<rect x="3.5" y="3" width="13" height="14" rx="3" />
						<path d="M13.25 5.5v9" />
					</svg>
				</button>

				<button
					type="button"
					class={iconButtonClass}
					aria-label="More problem actions"
					aria-haspopup="menu"
					aria-expanded={menuOpen}
					title="More problem actions"
					on:click={() => (menuOpen = !menuOpen)}
				>
					<svg viewBox="0 0 16 16" aria-hidden="true" class="h-4 w-4 fill-current">
						<circle cx="8" cy="4" r="1.15" />
						<circle cx="8" cy="8" r="1.15" />
						<circle cx="8" cy="12" r="1.15" />
					</svg>
				</button>

				{#if menuOpen}
					<div
						id="problem-actions-menu"
						role="menu"
						class="absolute top-[calc(100%+6px)] right-0 z-30 w-36 overflow-hidden rounded-lg border-[0.5px] border-[rgba(28,26,22,0.16)] bg-white py-1 shadow-lg shadow-stone-950/10"
						transition:fade={{ duration: 100 }}
					>
						<button
							type="button"
							role="menuitem"
							class={menuItemClass}
							disabled={lintBoxes.length === 0 || allOpen}
							on:click={openAllCards}
						>
							Open All
						</button>
						<button
							type="button"
							role="menuitem"
							class={menuItemClass}
							disabled={lintBoxes.length === 0 || openSet.size === 0}
							on:click={closeAllCards}
						>
							Close All
						</button>
						<div class="my-1 h-px bg-[rgba(28,26,22,0.09)]"></div>
						<button
							type="button"
							role="menuitem"
							class={dangerMenuItemClass}
							disabled={lintBoxes.length === 0}
							on:click={requestIgnoreAll}
						>
							Ignore All
						</button>
					</div>
				{/if}
			</div>
		</header>

		{#if showLintKindCounts && visibleLintKindEntries.length > 0}
			<div
				class="grid grid-cols-3 gap-x-2 gap-y-[7px] overflow-hidden border-b-[0.5px] border-[rgba(28,26,22,0.09)] px-[18px] pb-3"
				aria-label="Problem lint kinds"
			>
				{#each visibleLintKindEntries as [key, lintKindStyle]}
					<div
						class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1 whitespace-nowrap text-[10px] font-medium text-stone-500"
					>
						<span
							class={`inline-flex h-[9px] w-[9px] shrink-0 items-center justify-center rounded-full ${lintKindStyle.haloClass}`}
						>
							<span class={`h-[7px] w-[7px] rounded-full ${lintKindStyle.dotClass}`}></span>
						</span>
						<span class="overflow-hidden text-ellipsis">{lintKindStyle.label}</span>
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
					{@const id = lintBox.lint.context_hash}
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
	</div>
</aside>

{#if showIgnoreConfirm}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-stone-950/35 px-4"
		role="presentation"
		transition:fade={{ duration: 120 }}
		on:click={handleIgnoreBackdropClick}
	>
		<div
			class="w-full max-w-[340px] rounded-lg border-[0.5px] border-[rgba(28,26,22,0.16)] bg-white p-4 text-stone-950 shadow-2xl shadow-stone-950/20"
			role="dialog"
			aria-modal="true"
			aria-labelledby="ignore-all-title"
			tabindex="-1"
		>
			<h3 id="ignore-all-title" class="!m-0 !p-0 text-[15px] leading-5 font-semibold">
				Ignore all problems?
			</h3>
			<p class="mt-2 mb-4 text-[13px] leading-5 text-stone-600">
				This will ignore {problemCountLabel} in the current document.
			</p>
			<div class="flex justify-end gap-2">
				<button
					type="button"
					class="h-8 rounded-md border-[0.5px] border-stone-300 bg-linear-to-b from-white to-stone-50 px-3 text-[13px] font-medium text-stone-700 shadow-sm shadow-stone-950/5"
					on:click={cancelIgnoreAll}
				>
					Cancel
				</button>
				<button
					type="button"
					class="h-8 rounded-md border-[0.5px] border-red-700 bg-red-700 px-3 text-[13px] font-medium text-white shadow-sm shadow-red-950/10 disabled:opacity-50"
					disabled={lintBoxes.length === 0}
					on:click={confirmIgnoreAll}
				>
					Ignore All
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	aside :global(code) {
		font-family: inherit;
		font-size: inherit;
	}
</style>

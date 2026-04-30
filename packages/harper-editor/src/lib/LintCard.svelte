<script lang="ts">
import { slide } from 'svelte/transition';
import { displayCategoryFor, LINT_CATEGORIES, suggestionText } from './editorDisplay.js';
import type { UnpackedLint, UnpackedSuggestion } from './types.js';

export let lint: UnpackedLint;
export let open = false;
export let active = false;
export let onToggleOpen: () => void;
export let focusError: () => void;
export let onActivate: () => void = () => {};
export let onApply: (s: UnpackedSuggestion) => void;
export let onIgnore: () => void | Promise<void> = () => {};
export let snippet: {
	prefix: string;
	problem: string;
	suffix: string;
	prefixEllipsis: boolean;
	suffixEllipsis: boolean;
};

let cardEl: HTMLDivElement | null = null;

const iconClass =
	'h-3.5 w-3.5 fill-none stroke-current stroke-[1.6] [stroke-linecap:round] [stroke-linejoin:round]';
const baseSuggestionClass =
	'h-[26px] max-w-full overflow-hidden text-ellipsis rounded-full px-[11px] text-[12.5px] font-semibold [font-family:var(--harper-editor-font-family)]';

$: category = displayCategoryFor(lint.lint_kind);
$: categoryConfig = LINT_CATEGORIES[category];
$: if (open && cardEl != null) {
	requestAnimationFrame(() => {
		const scroller = cardEl?.closest('[data-problems-scroller]');
		if (!(scroller instanceof HTMLElement) || cardEl == null) {
			return;
		}

		const top = cardEl.offsetTop;
		const bottom = top + cardEl.offsetHeight;
		const viewTop = scroller.scrollTop;
		const viewBottom = viewTop + scroller.clientHeight;

		if (top < viewTop + 8) {
			scroller.scrollTo({ top: Math.max(0, top - 14), behavior: 'smooth' });
		} else if (bottom > viewBottom - 8) {
			scroller.scrollTo({ top: bottom - scroller.clientHeight + 14, behavior: 'smooth' });
		}
	});
}

function handleFocus() {
	onActivate();
	focusError?.();
}
</script>

<div
	bind:this={cardEl}
	role="group"
	class={`shrink-0 overflow-hidden rounded-[10px] border-[0.5px] bg-white shadow-sm shadow-stone-950/5 transition-[box-shadow,border-color] duration-150 ${
		active ? categoryConfig.activeClass : 'border-[rgba(28,26,22,0.14)]'
	}`}
	on:mouseenter={onActivate}
>
	<button
		type="button"
		class="m-0 flex min-h-8 w-full items-center gap-2 border-0 bg-transparent px-3 py-2 text-left text-inherit"
		aria-expanded={open}
		on:click={onToggleOpen}
	>
		<span
			class={`inline-flex h-[11px] w-[11px] shrink-0 items-center justify-center rounded-full ${categoryConfig.haloClass}`}
		>
			<span class={`h-[7px] w-[7px] rounded-full ${categoryConfig.dotClass}`}></span>
		</span>
		<span class="text-[12.5px] leading-[1.1] font-semibold text-stone-950">
			{categoryConfig.label}
		</span>
		<span
			class={`ml-auto inline-flex shrink-0 text-stone-500 transition-transform duration-150 ${
				open ? 'rotate-180' : ''
			}`}
		>
			<svg viewBox="0 0 16 16" aria-hidden="true" class={iconClass}>
				<path d="M4 6 8 10 12 6" />
			</svg>
		</span>
	</button>

	{#if open}
		<div class="flex flex-col gap-2.5 px-3 pt-0.5 pb-3" in:slide={{ duration: 130 }} out:slide={{ duration: 130 }}>
			<button
				type="button"
				class="m-0 flex w-full flex-col border-0 bg-transparent p-0 text-left text-[13px] leading-[1.4] font-medium text-stone-950"
				on:click={handleFocus}
			>
				<span>{@html lint.message_html}</span>
			</button>

			<button
				type="button"
				class="m-0 block max-h-[84px] w-full overflow-hidden rounded-[7px] border-[0.5px] border-[rgba(28,26,22,0.09)] bg-[#fbfaf6] px-3 py-2.5 text-left text-xs leading-[1.45] text-stone-700 [font-family:var(--harper-editor-font-family)]"
				on:click={handleFocus}
				aria-label="Focus problem in editor"
			>
				<span class="text-stone-500">
					{snippet.prefixEllipsis ? '...' : ''}{snippet.prefix}
				</span>
				<mark
					class={`rounded-[3px] px-0.5 font-semibold ${categoryConfig.softClass} ${categoryConfig.textClass}`}
					>{snippet.problem}</mark
				>
				<span class="text-stone-500">
					{snippet.suffix}{snippet.suffixEllipsis ? '...' : ''}
				</span>
			</button>

			<div class="flex items-center justify-end gap-2">
				{#if lint.suggestions && lint.suggestions.length > 0}
					<div class="flex flex-1 flex-wrap justify-end gap-1.5">
						{#each lint.suggestions as suggestion, i}
							<button
								type="button"
								class={`${baseSuggestionClass} ${
									i === 0
										? `border-transparent ${categoryConfig.softClass} ${categoryConfig.textClass} shadow-none`
										: 'border-[0.5px] border-stone-300 bg-linear-to-b from-white to-stone-50 text-stone-950 shadow-sm shadow-stone-950/5'
								}`}
								title={`Replace with "${suggestionText(suggestion)}"`}
								on:click={(event) => {
									event.stopPropagation();
									onApply?.(suggestion);
								}}
							>
								{suggestionText(suggestion)}
							</button>
						{/each}
					</div>
				{:else}
					<span class="mr-auto text-xs text-stone-400">No suggestions available.</span>
				{/if}

				<button
					type="button"
					class="h-[26px] shrink-0 border-0 bg-transparent px-1 text-[12.5px] font-medium text-stone-500 shadow-none"
					on:click={(event) => {
						event.stopPropagation();
						onIgnore();
					}}
				>
					Ignore
				</button>
			</div>
		</div>
	{/if}
</div>

<script lang="ts">
import { onMount, tick } from 'svelte';
import Button from './Button.svelte';
import ChevronLeftIcon from './icons/ChevronLeftIcon.svelte';
import ChevronRightIcon from './icons/ChevronRightIcon.svelte';
import Panel from './Panel.svelte';

/**
 * Slide deck shell: `slideProgress` runs from 0 (first slide) to 1 (last slide)
 * and controls navigation visibility. The default slot holds slide-specific
 * content; the `outside` slot renders below the clipped panel. `onBack` and
 * `onNext` synchronously update the parent's active slide before focus moves.
 */
export let title: string;
export let lede: string;
export let slideProgress: number;
export let onBack: () => void;
export let onNext: () => void;

let heading: HTMLElement;

onMount(() => {
	window.addEventListener('keydown', handleKeydown);
	return () => window.removeEventListener('keydown', handleKeydown);
});

/** Move within the walkthrough bounds, then focus the new heading after rendering. */
async function changeStep(direction: -1 | 1) {
	if (direction === -1 ? slideProgress <= 0 : slideProgress >= 1) return;
	if (direction === -1) onBack();
	else onNext();
	await tick();
	heading?.focus();
}

/** Keep navigation shortcuts out of editors and controls, including those inside shadow roots. */
function handleKeydown(event: KeyboardEvent) {
	if (
		event.defaultPrevented ||
		event.isComposing ||
		event.repeat ||
		event.altKey ||
		event.ctrlKey ||
		event.metaKey ||
		event.shiftKey ||
		event
			.composedPath()
			.some(
				(target) =>
					target instanceof Element &&
					target.closest(
						'input, textarea, select, button, a, summary, video, [contenteditable]:not([contenteditable="false"]), [role="button"], [role="textbox"]',
					),
			)
	) {
		return;
	}

	if (event.key === 'ArrowRight' || event.key === 'Enter') {
		event.preventDefault();
		void changeStep(1);
	} else if (event.key === 'ArrowLeft') {
		event.preventDefault();
		void changeStep(-1);
	}
}
</script>

<main class="grid h-dvh grid-cols-1 place-items-center overflow-y-auto bg-[#f1ede5] px-6 py-6 text-[var(--marketing-ink)] scheme-light [scrollbar-gutter:stable_both-edges] sm:py-12">
	<!-- Viewport-based width keeps the panel unchanged when reserving scrollbar gutters. -->
	<div class="relative w-[calc(100vw-3rem)] max-w-[46rem]">
		<div class="overflow-hidden rounded-lg border border-black/15 bg-[#fbfaf7] shadow-sm">
			<div
				class="h-1 bg-black/5"
				role="progressbar"
				aria-label="Slide deck progress"
				aria-valuemin={0}
				aria-valuemax={100}
				aria-valuenow={Math.round(slideProgress * 100)}
			>
				<div
					class="h-full bg-linear-to-b from-[var(--marketing-amber-soft)] to-[var(--marketing-amber)] transition-[width] duration-200 ease-out motion-reduce:transition-none"
					style:width={`${slideProgress * 100}%`}
				></div>
			</div>

			<div class="p-3 sm:p-6">
				<Panel class="border-black/10! shadow-sm">
					<div class="min-h-76 p-4 {slideProgress === 1 ? 'sm:p-8' : 'sm:p-7'}">
						{#key slideProgress}
							<div>
								<svelte:element
									this={slideProgress === 0 ? 'h1' : 'h2'}
									bind:this={heading}
									tabindex="-1"
									id="step-heading"
									class="m-0! p-0! font-[family-name:Domine,serif] font-semibold! text-3xl! leading-tight! tracking-normal! text-pretty focus:outline-none sm:text-4xl!"
								>
									{title}
								</svelte:element>
								<p class="m-0! max-w-lg font-[family-name:Domine,serif] text-base font-medium leading-relaxed! text-pretty text-[var(--marketing-ink-2)] sm:text-lg {slideProgress === 1 ? 'mt-10!' : 'mt-5!'}">{lede}</p>
								<slot />
							</div>
						{/key}
					</div>
				</Panel>
			</div>

			<div class="flex items-center justify-end gap-2 border-t border-black/10 bg-[#fbfaf7] px-6 pt-3 pb-4">
				{#if slideProgress > 0}
					<Button size="sm" color="light" class="bg-white! text-gray-900! hover:bg-gray-100! motion-reduce:transition-none" aria-label="Back" on:click={() => changeStep(-1)}>
						<ChevronLeftIcon className="size-4" />
					</Button>
				{/if}
				{#if slideProgress < 1}
					<Button size="sm" color="primary" class="bg-primary-600! text-white! hover:bg-primary-700! focus:ring-primary-300! motion-reduce:transition-none" on:click={() => changeStep(1)}>
						<span>Next</span>
						<ChevronRightIcon className="size-5" />
					</Button>
				{/if}
			</div>
		</div>

		<!-- Keep supplementary content outside the clipped panel and its centering calculation. -->
		<slot name="outside" />
	</div>
</main>

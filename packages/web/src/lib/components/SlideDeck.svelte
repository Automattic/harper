<script lang="ts">
import { Button, ChevronLeftIcon, ChevronRightIcon, Panel } from 'components';
import { onMount, tick } from 'svelte';

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
		<div class="overflow-hidden rounded-[10px] border-[0.5px] border-[rgba(28,26,22,0.14)] bg-[#fbfaf7] shadow-[inset_0_0.5px_0_rgba(255,255,255,0.8),0_1px_2px_rgba(28,26,22,0.06)]">
			<div
				class="h-[3px] bg-[rgba(28,26,22,0.07)]"
				role="progressbar"
				aria-label="Slide deck progress"
				aria-valuemin={0}
				aria-valuemax={100}
				aria-valuenow={Math.round(slideProgress * 100)}
			>
				<div
					class="h-full bg-[linear-gradient(180deg,var(--marketing-amber-soft),var(--marketing-amber))] transition-[width] duration-[240ms] ease-out motion-reduce:transition-none"
					style:width={`${slideProgress * 100}%`}
				></div>
			</div>

			<div class="p-6 [@media(max-width:400px)]:p-3">
				<Panel class="border-[rgba(28,26,22,0.09)]! shadow-[inset_0_0.5px_0_rgba(255,255,255,0.8)]">
					<div class="min-h-[19rem] [@media(max-width:400px)]:p-4 {slideProgress === 1 ? 'p-8' : 'p-7'}">
						{#key slideProgress}
							<div>
								<svelte:element
									this={slideProgress === 0 ? 'h1' : 'h2'}
									bind:this={heading}
									tabindex="-1"
									id="step-heading"
									class="m-0! p-0! font-[family-name:Domine,serif] font-[650]! leading-[1.08]! tracking-normal! text-pretty focus:[outline:none] {slideProgress === 1 ? 'text-[1.9rem]! sm:text-[2.5rem]!' : slideProgress === 0 ? 'text-[clamp(1.9rem,4vw,2.5rem)]!' : 'text-[clamp(1.7rem,3.4vw,2.2rem)]!'}"
								>
									{title}
								</svelte:element>
								<p class="font-[family-name:Domine,serif] font-[550] leading-[1.6]! text-pretty text-[var(--marketing-ink-2)] {slideProgress === 1 ? 'm-[2.5rem_0_0]!' : 'm-[1.25rem_0_0]! max-w-[31rem]'} {slideProgress === 1 ? 'text-[1.08rem] sm:text-[1.2rem]' : slideProgress === 0 ? 'text-[1.08rem]' : 'text-[1.02rem]'}">{lede}</p>
								<slot />
							</div>
						{/key}
					</div>
				</Panel>
			</div>

			<div class="flex items-center justify-end gap-2 border-t-[0.5px] border-t-[rgba(28,26,22,0.09)] bg-[rgba(251,250,247,0.88)] px-6 pt-3 pb-4">
				{#if slideProgress > 0}
					<Button size="sm" color="light" class="border-gray-200! bg-[#fff]! text-gray-900! hover:bg-gray-100! motion-reduce:transition-none" aria-label="Back" on:click={() => changeStep(-1)}>
						<ChevronLeftIcon className="size-[14px]" />
					</Button>
				{/if}
				{#if slideProgress < 1}
					<Button size="sm" color="primary" class="bg-primary-600! text-[#fff]! hover:bg-primary-700! focus:ring-primary-300! motion-reduce:transition-none" on:click={() => changeStep(1)}>
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

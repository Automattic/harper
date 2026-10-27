<script lang="ts">
import { onMount, tick } from 'svelte';
import Button from './Button.svelte';
import ChevronLeftIcon from './icons/ChevronLeftIcon.svelte';
import ChevronRightIcon from './icons/ChevronRightIcon.svelte';
import Panel from './Panel.svelte';

/**
 * Slide deck shell: `slideProgress` runs from 0 (first slide) to 1 (last slide)
 * and controls navigation visibility. The default slot holds slide-specific
 * content; the `outside` slot renders below the clipped panel. Set `showAside`
 * to place the `aside` slot to the right of the text on desktop or below it on
 * mobile. `onBack` and `onNext` synchronously update the parent's active slide
 * before focus moves.
 * `nextDisabled` blocks forward navigation through both the button and keyboard
 * shortcuts without preventing users from going back. Forward navigation plays a
 * diagonal flash across the slide content unless the user prefers reduced motion.
 */
export let title: string;
export let lede: string;
export let slideProgress: number;
export let onBack: () => void;
export let onNext: () => void;
export let nextDisabled = false;
export let showAside = false;

let heading: HTMLElement;

/**
 * Number of flashes played. Each increment recreates the flash overlay through
 * `{#key}`, which restarts its CSS animation. Pointer presses on Next increment it
 * on press down; keyboard navigation increments it when the step changes.
 */
let flashCount = 0;

onMount(() => {
	window.addEventListener('keydown', handleKeydown);
	return () => window.removeEventListener('keydown', handleKeydown);
});

function canAdvance() {
	return slideProgress < 1 && !nextDisabled;
}

/**
 * Move within the walkthrough bounds, then focus the new heading after rendering.
 * `flash` is false for pointer clicks because their press down already played it.
 */
async function changeStep(direction: -1 | 1, flash = direction === 1) {
	if (direction === -1 ? slideProgress <= 0 : !canAdvance()) return;
	if (direction === -1) onBack();
	else {
		onNext();
		if (flash) flashCount += 1;
	}
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
	<div class="relative w-[calc(100vw-3rem)] max-w-[64rem]">
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

			<div class="relative overflow-hidden p-3 sm:p-6">
				<Panel class="border-black/10! shadow-sm">
					<div class="grid min-h-112 p-6 sm:p-10">
						{#key slideProgress}
							<div class={showAside ? 'grid items-center gap-10 md:grid-cols-[minmax(0,1fr)_minmax(0,16rem)]' : ''}>
								<div class="min-w-0 self-start">
									<svelte:element
										this={slideProgress === 0 ? 'h1' : 'h2'}
										bind:this={heading}
										tabindex="-1"
										id="step-heading"
										class="m-0! p-0! font-[family-name:Domine,serif] font-semibold! leading-tight! tracking-normal! text-pretty focus:outline-none text-4xl! sm:text-5xl!"
									>
										{title}
									</svelte:element>
									<p class="mx-0! mb-0! font-[family-name:Domine,serif] font-medium leading-relaxed! text-pretty text-[var(--marketing-ink-2)] max-w-2xl text-lg sm:text-xl mt-6!">{lede}</p>
									<slot />
								</div>
								{#if showAside}
									<div class="flex min-w-0 items-center justify-center self-stretch py-3 md:px-3">
										<slot name="aside" />
									</div>
								{/if}
							</div>
						{/key}
					</div>
				</Panel>

				{#key flashCount}
					{#if flashCount > 0}
						<div class="slide-flash pointer-events-none absolute inset-0 z-10" aria-hidden="true"></div>
					{/if}
				{/key}
			</div>

			<div class="flex items-center justify-end gap-2 border-t border-black/10 bg-[#fbfaf7] px-6 pt-3 pb-4">
				{#if slideProgress > 0}
					<Button size="md" color="white" class="bg-white! text-gray-900! hover:bg-gray-100! motion-reduce:transition-none" aria-label="Back" on:click={() => changeStep(-1)}>
						<ChevronLeftIcon className="size-4" />
					</Button>
				{/if}
				{#if slideProgress < 1}
					<Button size="md" color="primary" class="bg-primary-600! text-white! hover:bg-primary-700! focus:ring-primary-300! motion-reduce:transition-none" disabled={nextDisabled}
						onpointerdown={(event: PointerEvent) => {
							if (event.button === 0 && canAdvance()) flashCount += 1;
						}}
						onclick={(event: MouseEvent) => changeStep(1, event.detail === 0)}
					>
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

<script lang="ts">
import { Button, ChevronLeftIcon, ChevronRightIcon, Link, Panel, Textarea } from 'components';
import { onMount, tick } from 'svelte';
import AppLogoTile from '$lib/marketing/AppLogoTile.svelte';
import { marketingLinks } from '$lib/marketing/data';

const steps = [
	{
		title: 'Thanks for Installing Harper!',
		lede: 'Writing is hard. Writing well is harder. Now that Harper is available on most websites you visit, it should be a little easier.',
	},
	{
		title: 'Launch Any Site',
		lede: 'Harper plugs right into the page of any site you visit.',
	},
	{
		title: 'Type Naturally',
		lede: 'Use your browser just like you always do. Harper will underline any incorrect grammar while you type.',
	},
	{
		title: 'Review & Accept',
		lede: "Click on any underlined text to review Harper's suggestions.",
	},
	{
		title: 'Give It a Whirl!',
		lede: 'Use the text box below to try Harper out.',
	},
	{
		title: 'Join Us',
		lede: 'Harper is a community effort. If you encounter a bug or want to request a feature, you can do so by opening an issue on GitHub or joining our Discord server.',
	},
];

const communityLinks = [
	{ id: 'github', label: 'GitHub', href: `${marketingLinks.github}/issues` },
	{ id: 'discord', label: 'Discord', href: marketingLinks.discord },
];

let step = 0;
let drawerOpen = false;
let demoText =
	'Ths is an text box you can type in.\n\nany other site on the web will work the the same!';
let heading: HTMLElement;

onMount(() => {
	window.addEventListener('keydown', handleKeydown);
	return () => window.removeEventListener('keydown', handleKeydown);
});

/** Move one card within the walkthrough bounds, then focus its heading after rendering. */
async function changeStep(direction: -1 | 1) {
	const next = Math.max(0, Math.min(steps.length - 1, step + direction));
	if (next === step) return;
	step = next;
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

<svelte:head>
	<title>Thanks for Installing Harper!</title>
</svelte:head>

<main class="grid h-dvh grid-cols-1 place-items-center overflow-y-auto bg-[#f1ede5] px-6 py-6 text-[var(--marketing-ink)] scheme-light [scrollbar-gutter:stable_both-edges] sm:py-12">
	<!-- Viewport-based width keeps the panel unchanged when reserving scrollbar gutters. -->
	<div class="relative w-[calc(100vw-3rem)] max-w-[46rem]">
		<div class="overflow-hidden rounded-[10px] border-[0.5px] border-[rgba(28,26,22,0.14)] bg-[#fbfaf7] shadow-[inset_0_0.5px_0_rgba(255,255,255,0.8),0_1px_2px_rgba(28,26,22,0.06)]">
			<div
				class="h-[3px] bg-[rgba(28,26,22,0.07)]"
				role="progressbar"
				aria-label="Installation walkthrough progress"
				aria-valuemin={0}
				aria-valuemax={steps.length}
				aria-valuenow={step + 1}
			>
				<div
					class="h-full bg-[linear-gradient(180deg,var(--marketing-amber-soft),var(--marketing-amber))] transition-[width] duration-[240ms] ease-out motion-reduce:transition-none"
					style:width={`${((step + 1) / steps.length) * 100}%`}
				></div>
			</div>

			<div class="p-6 [@media(max-width:400px)]:p-3">
				<Panel class="border-[rgba(28,26,22,0.09)]! shadow-[inset_0_0.5px_0_rgba(255,255,255,0.8)]">
					<div class="min-h-[19rem] [@media(max-width:400px)]:p-4 {step === 5 ? 'p-8' : 'p-7'}">
						{#key step}
							<div>
								<svelte:element
									this={step === 0 ? 'h1' : 'h2'}
									bind:this={heading}
									tabindex="-1"
									id="step-heading"
									class="m-0! p-0! font-[family-name:Domine,serif] font-[650]! leading-[1.08]! tracking-normal! text-pretty focus:[outline:none] {step === 5 ? 'text-[1.9rem]! sm:text-[2.5rem]!' : step === 0 ? 'text-[clamp(1.9rem,4vw,2.5rem)]!' : 'text-[clamp(1.7rem,3.4vw,2.2rem)]!'}"
								>
									{steps[step].title}
								</svelte:element>
								<p class="font-[family-name:Domine,serif] font-[550] leading-[1.6]! text-pretty text-[var(--marketing-ink-2)] {step === 5 ? 'm-[2.5rem_0_0]!' : 'm-[1.25rem_0_0]! max-w-[31rem]'} {step === 5 ? 'text-[1.08rem] sm:text-[1.2rem]' : step === 0 ? 'text-[1.08rem]' : 'text-[1.02rem]'}">{steps[step].lede}</p>

								{#if step === 1}
									<div class="mt-[1.9rem] flex flex-wrap gap-[0.55rem] [&>span]:bg-[#fff]!">
										{#each ['gmail', 'notion', 'github', 'slack', 'wordpress', 'reddit'] as id}
											<AppLogoTile {id} size={32} />
										{/each}
									</div>
								{:else if step === 2 || step === 3}
									<video
										class="mt-6 block w-full rounded-[6px] border-[0.5px] border-[rgba(28,26,22,0.14)]"
										autoplay
										muted
										loop
										playsinline
										src={step === 2 ? '/videos/step-2.mp4' : '/videos/step-3.mp4'}
										aria-label={steps[step].lede}
									>
										Your browser does not support embedded videos.
									</video>
								{:else if step === 4}
									<Textarea
										rows={7}
										bind:value={demoText}
										class="mt-[1.1rem] block box-border w-full resize-y rounded-[6px]! border-[0.5px]! border-[rgba(28,26,22,0.14)]! bg-[#fbfaf7]! px-4 py-[0.9rem] font-[family-name:Atkinson_Hyperlegible,sans-serif] text-[1.05rem] leading-[1.6] text-[var(--marketing-ink)]! [box-shadow:inset_0_1px_2px_rgba(28,26,22,0.05)]! focus:border-[#2a6bd8]! focus:outline-2! focus:outline-solid! focus:outline-[rgba(42,107,216,0.35)]! motion-reduce:transition-none"
										aria-label="Try Harper here"
									/>

									<div class="mt-[1.2rem] overflow-hidden rounded-[6px] border-[0.5px] border-[rgba(28,26,22,0.09)] bg-[#faf8f3]">
										<button
											type="button"
											id="troubleshooting-toggle"
											class="flex w-full cursor-pointer items-center justify-between gap-4 px-[0.9rem] py-3 text-left text-[0.92rem] font-bold focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-solid focus-visible:outline-[#2a6bd8]"
											aria-expanded={drawerOpen}
											aria-controls="troubleshooting-body"
											on:click={() => (drawerOpen = !drawerOpen)}
										>
											<span class="whitespace-nowrap">Nothing is Happening?</span>
											<span class="font-[family-name:JetBrains_Mono,monospace] text-[0.95rem] font-normal text-[var(--marketing-amber)]" aria-hidden="true">{drawerOpen ? '−' : '+'}</span>
										</button>
									</div>
								{:else if step === 5}
									<div class="mt-8 flex flex-col gap-3">
										{#each communityLinks as link}
											<Link
												href={link.href}
												target="_blank"
												rel="noopener noreferrer"
												aria-label={`${link.label} (opens in a new tab)`}
												class="flex items-center gap-3 rounded-[6px] border-[0.5px] border-[rgba(28,26,22,0.09)] bg-[#faf8f3] px-4 py-3 text-base font-bold text-[var(--marketing-ink)]! transition-colors hover:bg-[#f1ede5] hover:no-underline! focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-solid focus-visible:outline-[#2a6bd8] motion-reduce:transition-none [&>span:first-child]:bg-[#fff]!"
											>
												<AppLogoTile id={link.id} size={36} />
												<span>{link.label}</span>
												<ChevronRightIcon className="ml-auto size-4 shrink-0 text-[var(--marketing-amber)]" />
											</Link>
										{/each}
									</div>
								{/if}
							</div>
						{/key}
					</div>
				</Panel>
			</div>

			<div class="flex items-center justify-end gap-2 border-t-[0.5px] border-t-[rgba(28,26,22,0.09)] bg-[rgba(251,250,247,0.88)] px-6 pt-3 pb-4">
				{#if step > 0}
					<Button size="sm" color="light" class="border-gray-200! bg-[#fff]! text-gray-900! hover:bg-gray-100! motion-reduce:transition-none" aria-label="Back" on:click={() => changeStep(-1)}>
						<ChevronLeftIcon className="size-[14px]" />
					</Button>
				{/if}
				{#if step < steps.length - 1}
					<Button size="sm" color="primary" class="bg-primary-600! text-[#fff]! hover:bg-primary-700! focus:ring-primary-300! motion-reduce:transition-none" on:click={() => changeStep(1)}>
						<span>Next</span>
						<ChevronRightIcon className="size-5" />
					</Button>
				{/if}
			</div>
		</div>

		<!-- Keep the drawer outside the clipped panel and its centering calculation. -->
		{#if step === 4}
			<div id="troubleshooting-body" hidden={!drawerOpen} class="absolute inset-x-0 top-full mt-4 pb-6 sm:pb-12">
				{#if drawerOpen}
					<div
						role="region"
						aria-labelledby="troubleshooting-toggle"
						class="grid items-start gap-5 rounded-[6px] border-[0.5px] border-[rgba(28,26,22,0.09)] bg-[#fff] p-[0.9rem] sm:grid-cols-[minmax(0,1fr)_minmax(10rem,13rem)]"
					>
						<p class="m-0! text-[0.9rem] leading-[1.6]! text-[var(--marketing-ink-2)]">
							Harper will only enable itself automatically on sites we've tested before.
							<br /><br />
							If you work somewhere that isn't on our list of supported sites, you can enable the extension anyway by opening the Harper extension popup and clicking the power button.
							<br /><br />
							Alternatively, <Link class="text-[var(--marketing-amber)]! hover:text-[var(--marketing-ink)]!" href="/request-browser-support">let us know</Link> which sites you want us to support and we'll add it as soon as we can.
						</p>
						<img
							src="/images/chrome_extension_popup.png"
							alt="The Chrome extension's popup page"
							class="h-auto w-full rounded-[6px] border-[0.5px] border-[rgba(28,26,22,0.14)] object-contain"
						/>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</main>

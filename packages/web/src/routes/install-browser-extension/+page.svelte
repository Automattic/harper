<script lang="ts">
import { ChevronRightIcon, Link, SlideDeck, Textarea } from 'components';
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
</script>

<svelte:head>
	<title>Thanks for Installing Harper!</title>
</svelte:head>

<SlideDeck
	title={steps[step].title}
	lede={steps[step].lede}
	slideProgress={step / (steps.length - 1)}
	onBack={() => step--}
	onNext={() => step++}
>
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

	<svelte:fragment slot="outside">
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
	</svelte:fragment>
</SlideDeck>

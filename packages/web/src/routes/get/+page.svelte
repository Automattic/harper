<script lang="ts">
import { onMount } from 'svelte';
import {
	type Integration,
	integrationCategories,
	integrations,
	marketingLinks,
} from '$lib/marketing/data';
import IntegrationTile from '$lib/marketing/IntegrationTile.svelte';
import MarketingFooter from '$lib/marketing/MarketingFooter.svelte';
import MarketingHeader from '$lib/marketing/MarketingHeader.svelte';
import { liveVersions, loadLiveVersions } from '$lib/marketing/versions';

let activeCategory = 'all';
let query = '';

onMount(() => {
	void loadLiveVersions();
});

$: filtered = integrations.filter((integration) => {
	if (activeCategory === 'community' && !integration.community) {
		return false;
	}

	if (
		activeCategory !== 'all' &&
		activeCategory !== 'community' &&
		integration.category !== activeCategory
	) {
		return false;
	}

	const term = query.trim().toLowerCase();
	if (!term) {
		return true;
	}

	return [integration.name, integration.desc, integration.categoryLabel, integration.platform ?? '']
		.join(' ')
		.toLowerCase()
		.includes(term);
});

$: communityCount = integrations.filter((integration) => integration.community).length;

function ctaLabel(integration: Integration) {
	return integration.cta === 'install' ? 'Install' : 'View docs';
}

function clearFilters() {
	query = '';
	activeCategory = 'all';
}
</script>

<svelte:head>
	<title>Get Harper</title>
	<meta
		name="description"
		content="Install Harper for desktop apps, browsers, code editors, and developer workflows."
	/>
</svelte:head>

<div class="min-h-screen bg-[#f6f1e6] text-[#1c1a16] dark:bg-black dark:text-white">
	<MarketingHeader active="get" />

	<section class="border-b-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fdfbf5] px-10 py-16 pb-[4.25rem] text-center dark:border-white/10 dark:bg-black max-[860px]:px-4">
		<div class="mx-auto max-w-[58rem]">
			<h1 class="!mt-[0.85rem] !mb-0 py-0 !font-serif text-[clamp(3.2rem,7vw,3.5rem)] font-[650] leading-[1.02] tracking-normal text-inherit">
				Take Harper with you.
			</h1>
			<p class="mx-auto !mt-3 !mb-[2.2rem] max-w-[35rem] !font-serif text-[1.2rem] leading-[1.45] text-[#4a463e] dark:text-white/70">
				Use Harper in your favorite apps and browsers. Good grammar goes where you are.
			</p>
			<div class="mx-auto grid max-w-[47.5rem] grid-cols-2 gap-[0.85rem] text-left max-[640px]:grid-cols-1">
				{#each ['desktop', 'chrome'] as id}
					{@const integration = integrations.find((item) => item.id === id)}
					{#if integration}
						<a
							class="grid grid-cols-[2.5rem_1fr_auto] items-center gap-[0.9rem] rounded-xl border-[0.5px] border-[rgba(28,26,22,0.1)] bg-white px-[1.1rem] py-[0.9rem] !text-[#1c1a16] no-underline transition-[transform,box-shadow,border-color] duration-150 hover:-translate-y-px hover:border-[#b06a1b] hover:shadow-[0_10px_24px_-16px_rgba(28,26,22,0.16)] hover:no-underline dark:border-white/10 dark:bg-white/5 dark:!text-white dark:hover:border-primary-300 max-[640px]:grid-cols-[2.5rem_1fr] [&_em]:max-[640px]:col-start-2"
							href={integration.href}
						>
							<IntegrationTile {integration} size={40} />
							<span class="flex min-w-0 flex-col">
								<strong class="text-[0.94rem] leading-[1.25]">{integration.name}</strong>
								<small class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.8rem] leading-[1.4] text-[#807a6e] dark:text-white/55">
									{integration.desc}
									{#if $liveVersions[integration.id]}
										<span class="opacity-75 font-mono ml-1">({$liveVersions[integration.id]})</span>
									{:else if $liveVersions[integration.name]}
										<span class="opacity-75 font-mono ml-1">({$liveVersions[integration.name]})</span>
									{/if}
								</small>
							</span>
							<em class="whitespace-nowrap text-[0.78rem] font-extrabold text-[#b06a1b] not-italic dark:text-primary-300">{ctaLabel(integration)} →</em>
						</a>
					{/if}
				{/each}
			</div>
			<div class="mt-[1.1rem] text-[0.82rem] text-[#807a6e] dark:text-white/55">Or browse {integrations.length - 2} other integrations ↓</div>
		</div>
	</section>

	<section class="px-10 pt-[3.25rem] pb-20 max-[860px]:px-4" aria-label="Harper integrations">
		<div class="mx-auto grid max-w-[77.5rem] grid-cols-[14.5rem_1fr] items-start gap-9 max-[860px]:grid-cols-1">
			<aside class="sticky top-[5.6rem] flex flex-col gap-[1.1rem] max-[860px]:static">
				<label
					class="flex h-[2.4rem] items-center gap-[0.55rem] rounded-[0.65rem] border-[0.5px] border-[rgba(28,26,22,0.1)] bg-white px-3 transition-[border-color,outline-color] duration-150 focus-within:border-transparent focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-[#2a6bd8] dark:border-white/10 dark:bg-white/5"
					aria-label="Search integrations"
				>
					<svg class="size-[0.88rem] fill-none stroke-[#807a6e] stroke-[1.6] [stroke-linecap:round] dark:stroke-white/55" viewBox="0 0 16 16" aria-hidden="true">
						<circle cx="7" cy="7" r="4.5" />
						<path d="M10.5 10.5L14 14" />
					</svg>
					<input
						class="min-w-0 flex-1 appearance-none border-0! bg-transparent text-[0.84rem] text-[#1c1a16] shadow-none! outline-0! dark:text-white"
						bind:value={query}
						type="search"
						placeholder="Search..."
					/>
				</label>

				<nav class="flex flex-col gap-[0.15rem] max-[860px]:grid max-[860px]:grid-cols-2 max-[640px]:grid-cols-1" aria-label="Filter integrations">
					<button
						class={`flex cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent px-[0.6rem] py-[0.45rem] text-left text-[#4a463e] hover:bg-white hover:text-[#1c1a16] hover:shadow-[0_0_0_0.5px_rgba(28,26,22,0.1),0_1px_2px_rgba(28,26,22,0.04)] dark:text-white/70 dark:hover:bg-white/10 dark:hover:text-white ${
							activeCategory === 'all'
								? 'bg-white text-[#1c1a16] shadow-[0_0_0_0.5px_rgba(28,26,22,0.1),0_1px_2px_rgba(28,26,22,0.04)] dark:bg-white/10 dark:text-white'
								: ''
						}`}
						type="button"
						on:click={() => (activeCategory = 'all')}
					>
						<span class="flex-1 text-[0.84rem] font-[650]">All integrations</span><b class='text-[0.68rem] font-semibold text-[#807a6e] dark:text-white/55 [font-family:"JetBrains_Mono",monospace]'>{integrations.length}</b>
					</button>
					{#each integrationCategories as category}
						<button
							class={`flex cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent px-[0.6rem] py-[0.45rem] text-left text-[#4a463e] hover:bg-white hover:text-[#1c1a16] hover:shadow-[0_0_0_0.5px_rgba(28,26,22,0.1),0_1px_2px_rgba(28,26,22,0.04)] dark:text-white/70 dark:hover:bg-white/10 dark:hover:text-white ${
								activeCategory === category.id
									? 'bg-white text-[#1c1a16] shadow-[0_0_0_0.5px_rgba(28,26,22,0.1),0_1px_2px_rgba(28,26,22,0.04)] dark:bg-white/10 dark:text-white'
									: ''
							}`}
							type="button"
							on:click={() => (activeCategory = category.id)}
						>
							<span class="flex-1 text-[0.84rem] font-[650]">{category.label}</span><b class='text-[0.68rem] font-semibold text-[#807a6e] dark:text-white/55 [font-family:"JetBrains_Mono",monospace]'>{category.items.length}</b>
						</button>
					{/each}
					<button
						class={`flex cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent px-[0.6rem] py-[0.45rem] text-left text-[#4a463e] hover:bg-white hover:text-[#1c1a16] hover:shadow-[0_0_0_0.5px_rgba(28,26,22,0.1),0_1px_2px_rgba(28,26,22,0.04)] dark:text-white/70 dark:hover:bg-white/10 dark:hover:text-white ${
							activeCategory === 'community'
								? 'bg-white text-[#1c1a16] shadow-[0_0_0_0.5px_rgba(28,26,22,0.1),0_1px_2px_rgba(28,26,22,0.04)] dark:bg-white/10 dark:text-white'
								: ''
						}`}
						type="button"
						on:click={() => (activeCategory = 'community')}
					>
						<span class="flex-1 text-[0.84rem] font-[650]">From the community</span><b class='text-[0.68rem] font-semibold text-[#807a6e] dark:text-white/55 [font-family:"JetBrains_Mono",monospace]'>{communityCount}</b>
					</button>
				</nav>

				{#if query || activeCategory !== 'all'}
					<p class='!m-0 text-[0.7rem] text-[#807a6e] dark:text-white/55 [font-family:"JetBrains_Mono",monospace]'>
						{filtered.length} {filtered.length === 1 ? 'match' : 'matches'} ·
						<button class="cursor-pointer border-0 bg-transparent font-extrabold text-[#b06a1b] dark:text-primary-300" type="button" on:click={clearFilters}>Clear</button>
					</p>
				{/if}
			</aside>

			<div class="grid grid-cols-[repeat(auto-fill,minmax(20rem,1fr))] gap-3 max-[640px]:grid-cols-1">
				{#if filtered.length === 0}
					<div class="col-span-full rounded-[0.9rem] border-[0.5px] border-dashed border-[rgba(28,26,22,0.16)] bg-white px-6 py-[3.75rem] text-center dark:border-white/15 dark:bg-white/5">
						<h2 class="!mt-0 !mb-1.5 py-0 !font-serif text-[1.4rem]">No match for "{query}"</h2>
						<p class="!m-0 text-[#807a6e] dark:text-white/55">
							Don’t see your editor? <a class="font-extrabold !text-[#b06a1b] no-underline hover:no-underline dark:!text-primary-300" href={marketingLinks.github}>Help us build it →</a>
						</p>
					</div>
				{:else}
					{#each filtered as integration}
						<a
							class="grid grid-cols-[2.5rem_1fr_auto] items-center gap-[0.9rem] rounded-xl border-[0.5px] border-[rgba(28,26,22,0.1)] bg-white px-[1.1rem] py-[0.9rem] !text-[#1c1a16] no-underline transition-[transform,box-shadow,border-color] duration-150 hover:-translate-y-px hover:border-[#b06a1b] hover:shadow-[0_10px_24px_-16px_rgba(28,26,22,0.16)] hover:no-underline dark:border-white/10 dark:bg-white/5 dark:!text-white dark:hover:border-primary-300 max-[640px]:grid-cols-[2.5rem_1fr] [&_em]:max-[640px]:col-start-2"
							href={integration.href}
						>
						<IntegrationTile {integration} size={40} />
						<span class="flex min-w-0 flex-col">
							<strong class="text-[0.94rem] leading-[1.25]">{integration.name}</strong>
							<small class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.8rem] leading-[1.4] text-[#807a6e] dark:text-white/55">
								{integration.platform}
								{#if $liveVersions[integration.id]}
									<span class="opacity-75 font-mono ml-1">({$liveVersions[integration.id]})</span>
								{:else if $liveVersions[integration.name]}
									<span class="opacity-75 font-mono ml-1">({$liveVersions[integration.name]})</span>
								{/if}
							</small>
						</span>
						<em class="whitespace-nowrap text-[0.78rem] font-extrabold text-[#b06a1b] not-italic dark:text-primary-300">{ctaLabel(integration)} →</em>
						</a>
					{/each}
				{/if}
			</div>
		</div>
	</section>

	<MarketingFooter />
</div>

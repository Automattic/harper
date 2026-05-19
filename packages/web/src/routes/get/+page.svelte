<script lang="ts">
import {
	type Integration,
	integrationCategories,
	integrations,
	marketingLinks,
} from '$lib/marketing/data';
import IntegrationTile from '$lib/marketing/IntegrationTile.svelte';
import MarketingFooter from '$lib/marketing/MarketingFooter.svelte';
import MarketingHeader from '$lib/marketing/MarketingHeader.svelte';

let activeCategory = 'all';
let query = '';

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

<div class="marketing-page">
	<MarketingHeader active="get" />

	<section class="hero">
		<div class="hero-inner">
			<h1>Take Harper with you.</h1>
			<p>
				Use Harper in your favorite apps and browsers. Good grammar goes where you are.
			</p>
			<div class="hero-cards">
				{#each ['desktop', 'chrome'] as id}
					{@const integration = integrations.find((item) => item.id === id)}
					{#if integration}
						<a href={integration.href}>
							<IntegrationTile {integration} size={40} />
							<span>
								<strong>{integration.name}</strong>
								<small>{integration.desc}</small>
							</span>
							<em>{ctaLabel(integration)} →</em>
						</a>
					{/if}
				{/each}
			</div>
			<div class="hint">Or browse {integrations.length - 2} other integrations ↓</div>
		</div>
	</section>

	<section class="catalog" aria-label="Harper integrations">
		<div class="catalog-inner">
			<aside class="rail">
				<label class="search" aria-label="Search integrations">
					<svg viewBox="0 0 16 16" aria-hidden="true">
						<circle cx="7" cy="7" r="4.5" />
						<path d="M10.5 10.5L14 14" />
					</svg>
					<input bind:value={query} type="search" placeholder="Search..." />
				</label>

				<nav aria-label="Filter integrations">
					<button class:active={activeCategory === 'all'} type="button" on:click={() => (activeCategory = 'all')}>
						<span>All integrations</span><b>{integrations.length}</b>
					</button>
					{#each integrationCategories as category}
						<button
							class:active={activeCategory === category.id}
							type="button"
							on:click={() => (activeCategory = category.id)}
						>
							<span>{category.label}</span><b>{category.items.length}</b>
						</button>
					{/each}
					<button
						class:active={activeCategory === 'community'}
						type="button"
						on:click={() => (activeCategory = 'community')}
					>
						<span>From the community</span><b>{communityCount}</b>
					</button>
				</nav>

				{#if query || activeCategory !== 'all'}
					<p class="matches">
						{filtered.length} {filtered.length === 1 ? 'match' : 'matches'} ·
						<button type="button" on:click={clearFilters}>Clear</button>
					</p>
				{/if}
			</aside>

			<div class="grid">
				{#if filtered.length === 0}
					<div class="empty">
						<h2>No match for "{query}"</h2>
						<p>
							Don't see your editor? <a href={marketingLinks.github}>Help us build it →</a>
						</p>
					</div>
				{:else}
					{#each filtered as integration}
						<a class="card" href={integration.href}>
							<IntegrationTile {integration} size={40} />
							<span>
								<strong>{integration.name}</strong>
								<small>{integration.platform}</small>
							</span>
							<em>{ctaLabel(integration)} →</em>
						</a>
					{/each}
				{/if}
			</div>
		</div>
	</section>

	<MarketingFooter />
</div>

<style>
.marketing-page {
	min-height: 100vh;
	background: var(--marketing-page-bg);
	color: var(--marketing-ink);
	font-family: var(--marketing-sans);
}

.hero {
	border-bottom: 0.5px solid var(--marketing-line);
	background: #fdfbf5;
	padding: 4rem 2.5rem 4.25rem;
	text-align: center;
}

.hero-inner {
	max-width: 58rem;
	margin: 0 auto;
}

h1 {
	margin: 0.85rem 0 0;
	color: inherit;
	font-family: var(--marketing-display);
	font-size: clamp(3.2rem, 7vw, 3.5rem);
	font-weight: 650;
	line-height: 1.02;
	letter-spacing: 0;
}

.hero p {
	max-width: 35rem;
	margin: 0.75rem auto 2.2rem;
	color: var(--marketing-ink-2);
	font-family: var(--marketing-display);
	font-size: 1.2rem;
	line-height: 1.45;
}

.hero-cards {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.85rem;
	max-width: 47.5rem;
	margin: 0 auto;
	text-align: left;
}

.hero-cards a,
.card {
	display: grid;
	grid-template-columns: 2.5rem 1fr auto;
	align-items: center;
	gap: 0.9rem;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.75rem;
	background: #fff;
	color: var(--marketing-ink);
	padding: 0.9rem 1.1rem;
	text-decoration: none;
	transition:
		transform 120ms ease,
		box-shadow 120ms ease,
		border-color 120ms ease;
}

.hero-cards a:hover,
.card:hover {
	border-color: var(--marketing-amber);
	box-shadow: 0 10px 24px -16px rgba(28, 26, 22, 0.16);
	transform: translateY(-1px);
}

.hero-cards span,
.card span {
	display: flex;
	min-width: 0;
	flex-direction: column;
}

.hero-cards strong,
.card strong {
	font-size: 0.94rem;
	line-height: 1.25;
}

.hero-cards small,
.card small {
	overflow: hidden;
	color: var(--marketing-ink-3);
	font-size: 0.8rem;
	line-height: 1.4;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.hero-cards em,
.card em {
	color: var(--marketing-amber);
	font-size: 0.78rem;
	font-style: normal;
	font-weight: 800;
	white-space: nowrap;
}

.hint {
	margin-top: 1.1rem;
	color: var(--marketing-ink-3);
	font-size: 0.82rem;
}

.catalog {
	padding: 3.25rem 2.5rem 5rem;
}

.catalog-inner {
	display: grid;
	grid-template-columns: 14.5rem 1fr;
	align-items: flex-start;
	gap: 2.25rem;
	max-width: 77.5rem;
	margin: 0 auto;
}

.rail {
	position: sticky;
	top: 5.6rem;
	display: flex;
	flex-direction: column;
	gap: 1.1rem;
}

.search {
	display: flex;
	align-items: center;
	gap: 0.55rem;
	height: 2.4rem;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.65rem;
	background: #fff;
	padding: 0 0.75rem;
	transition:
		border-color 120ms ease,
		outline-color 120ms ease;
}

.search:focus-within {
	border-color: transparent;
	outline: 2px solid #2a6bd8;
	outline-offset: 2px;
}

.search svg {
	width: 0.88rem;
	height: 0.88rem;
	fill: none;
	stroke: var(--marketing-ink-3);
	stroke-linecap: round;
	stroke-width: 1.6;
}

.search input {
	-webkit-appearance: none;
	appearance: none;
	min-width: 0;
	flex: 1;
	border: 0 !important;
	background: transparent;
	box-shadow: none !important;
	color: var(--marketing-ink);
	font: inherit;
	font-size: 0.84rem;
	outline: 0 !important;
}

.search input:focus,
.search input:focus-visible {
	border: 0 !important;
	box-shadow: none !important;
	outline: 0 !important;
}

nav {
	display: flex;
	flex-direction: column;
	gap: 0.15rem;
}

nav button,
.matches button {
	border: 0;
	background: transparent;
	font: inherit;
}

nav button {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	border-radius: 0.5rem;
	color: var(--marketing-ink-2);
	cursor: pointer;
	padding: 0.45rem 0.6rem;
	text-align: left;
}

nav button:hover,
nav button.active {
	background: #fff;
	box-shadow:
		0 0 0 0.5px var(--marketing-line),
		0 1px 2px rgba(28, 26, 22, 0.04);
	color: var(--marketing-ink);
}

nav span {
	flex: 1;
	font-size: 0.84rem;
	font-weight: 650;
}

nav b {
	color: var(--marketing-ink-3);
	font-family: var(--marketing-mono);
	font-size: 0.68rem;
	font-weight: 600;
}

.matches {
	margin: 0;
	color: var(--marketing-ink-3);
	font-family: var(--marketing-mono);
	font-size: 0.7rem;
}

.matches button {
	color: var(--marketing-amber);
	cursor: pointer;
	font-weight: 800;
}

.grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
	gap: 0.75rem;
}

.empty {
	grid-column: 1 / -1;
	border: 0.5px dashed var(--marketing-line-strong);
	border-radius: 0.9rem;
	background: #fff;
	padding: 3.75rem 1.5rem;
	text-align: center;
}

.empty h2 {
	margin: 0 0 0.4rem;
	font-family: var(--marketing-display);
	font-size: 1.4rem;
}

.empty p {
	margin: 0;
	color: var(--marketing-ink-3);
}

.empty a {
	color: var(--marketing-amber);
	font-weight: 800;
	text-decoration: none;
}

@media (max-width: 860px) {
	.hero,
	.catalog {
		padding-inline: 1rem;
	}

	.catalog-inner {
		grid-template-columns: 1fr;
	}

	.rail {
		position: static;
	}

	nav {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}

@media (max-width: 640px) {
	.hero-cards,
	.grid,
	nav {
		grid-template-columns: 1fr;
	}

	.hero-cards a,
	.card {
		grid-template-columns: 2.5rem 1fr;
	}

	.hero-cards em,
	.card em {
		grid-column: 2;
	}
}
</style>

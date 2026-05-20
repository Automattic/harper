<script lang="ts">
import DiscordLogo from '$lib/components/DiscordLogo.svelte';
import GitHubLogo from '$lib/components/GitHubLogo.svelte';
import { onMount } from 'svelte';
import { marketingLinks } from './data';
import HarperMark from './HarperMark.svelte';
import MarketingDocSearch from './MarketingDocSearch.svelte';

export let active: 'home' | 'get' | 'desktop' | 'docs' | '' = '';

let compact = false;
let ctaPrimary = false;
let mobileOpen = false;

onMount(() => {
	const compactAfter = 36;
	const expandAtTop = 0;

	const update = () => {
		const y = window.scrollY;
		if (!compact && y > compactAfter) {
			compact = true;
		} else if (compact && y <= expandAtTop) {
			compact = false;
		}
		ctaPrimary = y > 360;
	};

	update();
	window.addEventListener('scroll', update, { passive: true });
	return () => window.removeEventListener('scroll', update);
});
</script>

<header class:compact class="marketing-header">
	<div class="inner">
		<a class="brand" href="/" aria-label="Harper home">
			<HarperMark size={compact ? 24 : 30} />
			<strong>Harper</strong>
		</a>

		<nav class="nav" aria-label="Marketing navigation">
			<a class:active={active === 'docs'} href="/docs/about">Documentation</a>
			<a class:active={active === 'get'} class:primary={ctaPrimary} class="header-cta" href="/get">
				<span>Get Harper</span>
				<svg viewBox="0 0 12 12" aria-hidden="true">
					<path d="M3 6h6M6.5 3.5L9 6 6.5 8.5" />
				</svg>
			</a>
		</nav>

		<div class="actions">
			<a class="icon-link" href={marketingLinks.github} aria-label="GitHub">
				<GitHubLogo />
			</a>
			<a class="icon-link" href={marketingLinks.discord} aria-label="Discord">
				<DiscordLogo />
			</a>
			<MarketingDocSearch />
			<button
				class="menu-button"
				type="button"
				aria-label="Toggle navigation"
				aria-expanded={mobileOpen}
				on:click={() => (mobileOpen = !mobileOpen)}
			>
				<span></span>
				<span></span>
			</button>
		</div>
	</div>

	{#if mobileOpen}
		<div class="mobile-panel">
			<a href="/docs/about" on:click={() => (mobileOpen = false)}>Documentation</a>
			<a href="/get" on:click={() => (mobileOpen = false)}>Get Harper</a>
			<a href="/desktop" on:click={() => (mobileOpen = false)}>Harper Desktop</a>
			<a href={marketingLinks.github} on:click={() => (mobileOpen = false)}>GitHub</a>
			<a href={marketingLinks.discord} on:click={() => (mobileOpen = false)}>Discord</a>
		</div>
	{/if}
</header>

<style>
.marketing-header {
	position: sticky;
	top: 0;
	z-index: 40;
	border-bottom: 0.5px solid transparent;
	background: rgba(246, 241, 230, 0.76);
	backdrop-filter: saturate(140%) blur(12px);
	-webkit-backdrop-filter: saturate(140%) blur(12px);
	font-family: var(--marketing-sans);
	transition:
		background 180ms ease,
		box-shadow 180ms ease,
		border-color 180ms ease;
}

.marketing-header.compact {
	border-color: var(--marketing-line);
	background: rgba(246, 241, 230, 0.9);
	box-shadow:
		0 1px 0 rgba(28, 26, 22, 0.04),
		0 8px 24px -20px rgba(28, 26, 22, 0.18);
}

.inner {
	display: grid;
	grid-template-columns: 1fr auto 1fr;
	align-items: center;
	gap: 1.5rem;
	max-width: 87.5rem;
	margin: 0 auto;
	padding: 1.15rem 2.5rem;
	transition: padding 220ms ease;
}

.compact .inner {
	padding-top: 0.32rem;
	padding-bottom: 0.32rem;
}

.brand {
	display: inline-flex;
	align-items: center;
	gap: 0.75rem;
	color: var(--marketing-ink);
	text-decoration: none;
	transition: gap 220ms ease;
}

.brand :global(.harper-mark) {
	transition:
		width 220ms ease,
		height 220ms ease;
}

.brand strong {
	font-size: 1.12rem;
	font-weight: 750;
	letter-spacing: 0;
	transition: font-size 220ms ease;
}

.nav {
	display: flex;
	align-items: center;
	gap: 0.75rem;
	justify-self: center;
	transition: gap 220ms ease;
}

.nav a,
.mobile-panel a {
	color: var(--marketing-ink);
	text-decoration: none;
}

.nav a {
	padding: 0.5rem 0.75rem;
	border-radius: 0.5rem;
	font-size: 0.94rem;
	font-weight: 650;
	transition:
		background 180ms ease,
		padding 220ms ease,
		font-size 220ms ease;
}

.nav a:hover {
	background: rgba(28, 26, 22, 0.05);
}

.nav a.active {
	font-weight: 750;
}

.header-cta {
	display: inline-flex;
	height: 2.125rem;
	align-items: center;
	gap: 0.45rem;
	border: 0.5px solid rgba(28, 26, 22, 0.22);
	border-radius: 999px;
	padding: 0 0.9rem;
	transition:
		background 180ms ease,
		border-color 180ms ease,
		color 180ms ease,
		height 220ms ease,
		padding 220ms ease,
		gap 220ms ease,
		transform 120ms ease;
}

.nav a.header-cta {
	padding: 0 0.9rem;
	transition:
		background 180ms ease,
		border-color 180ms ease,
		color 180ms ease,
		height 220ms ease,
		padding 220ms ease,
		gap 220ms ease,
		transform 120ms ease;
}

.header-cta:hover {
	transform: translateY(-0.5px);
}

.header-cta.primary {
	border-color: var(--marketing-amber);
	background: var(--marketing-amber);
	color: #fff;
}

.header-cta svg {
	width: 0.7rem;
	height: 0.7rem;
	fill: none;
	stroke: currentColor;
	stroke-linecap: round;
	stroke-linejoin: round;
	stroke-width: 1.5;
	transition:
		width 220ms ease,
		height 220ms ease;
}

.compact .nav a {
	padding-top: 0.42rem;
	padding-bottom: 0.42rem;
}

.compact .header-cta {
	height: 2rem;
	padding-inline: 0.82rem;
}

.compact .nav a.header-cta {
	padding-inline: 0.82rem;
}

.compact :global(.marketing-docsearch) {
	min-width: 10.25rem;
}

.compact :global(.marketing-docsearch .DocSearch-Button) {
	height: 2.125rem;
	padding: 0 0.62rem;
}

.compact :global(.marketing-docsearch .DocSearch-Button-Placeholder) {
	padding-right: 0.5rem;
	font-size: 0.92rem;
}

.compact :global(.marketing-docsearch .DocSearch-Search-Icon) {
	width: 0.95rem;
	height: 0.95rem;
}

.compact :global(.marketing-docsearch .DocSearch-Button-Keys) {
	min-width: 2rem;
	height: 1.18rem;
	font-size: 0.62rem;
}

.actions {
	display: flex;
	align-items: center;
	justify-content: flex-end;
	gap: 0.38rem;
	transition: gap 220ms ease;
}

:global(.marketing-docsearch) {
	transition: min-width 220ms ease;
}

:global(.marketing-docsearch .DocSearch-Button) {
	transition:
		background 180ms ease,
		color 180ms ease,
		height 220ms ease,
		padding 220ms ease,
		min-width 220ms ease;
}

:global(.marketing-docsearch .DocSearch-Button-Placeholder) {
	transition:
		padding 220ms ease,
		font-size 220ms ease;
}

:global(.marketing-docsearch .DocSearch-Search-Icon),
:global(.marketing-docsearch .DocSearch-Button-Keys) {
	transition:
		width 220ms ease,
		height 220ms ease,
		min-width 220ms ease,
		font-size 220ms ease;
}

.icon-link {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 2rem;
	height: 2rem;
	border-radius: 999px;
	color: var(--marketing-ink-2);
	text-decoration: none;
	transition:
		background 120ms ease,
		color 120ms ease,
		width 220ms ease,
		height 220ms ease;
}

.icon-link:hover {
	background: rgba(28, 26, 22, 0.05);
	color: var(--marketing-ink);
}

.icon-link :global(svg) {
	width: 1rem;
	height: 1rem;
	fill: currentColor;
	transition:
		width 220ms ease,
		height 220ms ease;
}

.menu-button {
	display: none;
	width: 2rem;
	height: 2rem;
	align-items: center;
	justify-content: center;
	flex-direction: column;
	gap: 0.25rem;
	border: 0;
	border-radius: 999px;
	background: transparent;
	color: var(--marketing-ink);
	transition:
		width 220ms ease,
		height 220ms ease,
		gap 220ms ease;
}

.menu-button span {
	width: 1rem;
	height: 1.5px;
	border-radius: 999px;
	background: currentColor;
	transition:
		width 220ms ease,
		height 220ms ease;
}

.mobile-panel {
	display: none;
}

@media (max-width: 900px) {
	.inner {
		grid-template-columns: 1fr auto;
		padding-inline: 1rem;
	}

	.nav,
	.actions :global(.marketing-docsearch),
	.icon-link {
		display: none;
	}

	.menu-button {
		display: inline-flex;
	}

	.mobile-panel {
		display: grid;
		gap: 0.25rem;
		padding: 0 1rem 1rem;
	}

	.mobile-panel a {
		border-radius: 0.75rem;
		background: rgba(255, 255, 255, 0.64);
		padding: 0.85rem 1rem;
		font-weight: 700;
	}
}

@media (prefers-reduced-motion: reduce) {
	.marketing-header,
	.inner,
	.brand,
	.brand :global(.harper-mark),
	.brand strong,
	.nav,
	.nav a,
	.header-cta,
	.header-cta svg,
	.actions,
	:global(.marketing-docsearch),
	:global(.marketing-docsearch .DocSearch-Button),
	:global(.marketing-docsearch .DocSearch-Button-Placeholder),
	:global(.marketing-docsearch .DocSearch-Search-Icon),
	:global(.marketing-docsearch .DocSearch-Button-Keys),
	.icon-link,
	.icon-link :global(svg),
	.menu-button,
	.menu-button span {
		transition-duration: 1ms;
	}
}

</style>

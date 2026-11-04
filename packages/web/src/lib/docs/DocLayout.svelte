<script lang="ts">
import { DiscordLogo, GitHubLogo } from 'components';
import { getContext, onMount, type Snippet } from 'svelte';
import { afterNavigate } from '$app/navigation';
import { page } from '$app/state';
import { navbar, sidebar, siteConfig, themeColor } from './config';
import DocSearch from './DocSearch.svelte';
import Sidebar from './Sidebar.svelte';
import type ThemeController from './ThemeController';
import { THEME_CONTEXT } from './ThemeController';

/** Shared site navigation around non-marketing content. */
interface Props {
	children: Snippet;
}

let { children }: Props = $props();
let mobileMenu: HTMLDialogElement;
let menuOpen = $state(false);
const theme = getContext<ThemeController>(THEME_CONTEXT);
const dark = $derived($theme === 'dark');
const entries = $derived(
	Object.entries(sidebar)
		.sort(([a], [b]) => b.length - a.length)
		.find(([prefix]) => page.url.pathname.startsWith(prefix))?.[1] ?? [],
);

onMount(() => {
	const media = window.matchMedia('(min-width: 1440px)');
	const closeOnDesktop = () => {
		if (media.matches) mobileMenu.close();
	};
	media.addEventListener('change', closeOnDesktop);
	return () => media.removeEventListener('change', closeOnDesktop);
});

afterNavigate(() => mobileMenu?.close());

/** Store an explicit choice through the root-owned theme controller. */
function toggleTheme() {
	theme.setTheme(dark ? 'light' : 'dark');
}
</script>

<div
	class="min-h-screen bg-(--docs-bg) text-(--docs-text) [font-family:Inter,ui-sans-serif,system-ui,sans-serif] leading-[1.75]
		[--docs-bg:#f8f8f8] [--docs-panel:#f2f2f2] [--docs-header-bg:#f6f6f6] [--docs-text:#3f3f46] [--docs-heading:#18181b] [--docs-muted:#71717a] [--docs-border:rgb(0_0_0/8%)] [--docs-link:#915808]
		dark:[--docs-bg:#18181b] dark:[--docs-panel:#161618] dark:[--docs-header-bg:#18181b] dark:[--docs-text:#d4d4d8] dark:[--docs-heading:#f4f4f5] dark:[--docs-muted:#a1a1aa] dark:[--docs-border:rgb(255_255_255/10%)] dark:[--docs-link:var(--docs-primary)] dark:scheme-dark
		[&_:where(a)]:docs-prose:text-inherit! [&_:where(a)]:docs-prose:no-underline [&_:where(a:hover)]:docs-prose:text-(--docs-link)! [&_button]:docs-prose:cursor-pointer
		[&_:focus-visible]:docs-prose:outline-2 [&_:focus-visible]:docs-prose:outline-(--docs-link) [&_:focus-visible]:docs-prose:outline-offset-4
		[&_[aria-current]]:docs-prose:text-(--docs-link)! [&_[aria-current]]:docs-prose:font-semibold"
	style:--docs-primary={themeColor.primary}
	style:--docs-primary-deep={themeColor.dark}
>
	<a class="fixed -top-[100px] left-4 z-[1100] bg-(--docs-bg) px-4 py-2 focus:top-2" href="#docs-main">Skip to content</a>
	<header class="fixed inset-x-0 top-0 z-100 border-b border-(--docs-border) bg-(--docs-header-bg)">
		<div class={`mx-auto flex h-14 max-w-[1440px] items-center gap-3 px-3 min-[950px]:h-[73px] min-[950px]:px-6! ${entries.length ? 'min-[950px]:pl-[calc(min(25vw,288px)+32px)]! min-[1440px]:pl-[calc(min(25vw,288px)+32px)]!'  : ''}`}>
			<button class="block h-10 w-8 text-[22px] min-[1440px]:hidden!" type="button" aria-label="Open navigation" aria-haspopup="dialog" aria-expanded={menuOpen} aria-controls="docs-mobile-menu" onclick={() => { mobileMenu.showModal(); menuOpen = true; }}>☰</button>
			<a class={`flex items-center gap-2 text-sm font-semibold leading-[1.4] ${entries.length ? 'min-[950px]:hidden!' : ''}`} href="/" aria-label={siteConfig.title}>
				<img class="flex-[0_0_32px]" src={siteConfig.logo} alt="" width="32" height="32" /><span class="max-[949px]:hidden!">{siteConfig.title}</span>
			</a>
			<DocSearch />
			<nav class="hidden items-center gap-5 whitespace-nowrap text-sm font-medium leading-[1.75] min-[1440px]:flex!" aria-label="Main navigation">
				{#each navbar as link}<a href={link.to} aria-current={page.url.pathname === link.to ? 'page' : undefined}>{link.title}</a>{/each}
			</nav>
			<div class="flex items-center gap-3.5 max-[479px]:gap-0!">
				<a class="size-5 max-[479px]:hidden!" href={siteConfig.github} aria-label="GitHub"><GitHubLogo /></a>
				<a class="size-5 max-[479px]:hidden!" href={siteConfig.discord} aria-label="Discord"><DiscordLogo /></a>
				<button class="size-8 text-xl" type="button" onclick={toggleTheme} aria-label="Dark mode" aria-pressed={dark}>{dark ? '☾' : '☀'}</button>
			</div>
		</div>
	</header>
	{#if entries.length}
		<aside class="fixed inset-y-0 left-0 z-[101] hidden w-[calc(max(0px,(100vw-1440px)/2)+min(25vw,288px))] overflow-y-auto border-r border-(--docs-border) bg-(--docs-panel) pr-6 pb-32 pl-[max(24px,calc((100vw-1440px)/2+24px))] min-[950px]:block!">
			<a class="sticky top-0 mb-4 flex min-h-[73px] items-center gap-2 border-b border-(--docs-border) bg-(--docs-panel) text-sm font-semibold leading-[1.4]" href="/">
				<img class="flex-[0_0_32px]" src={siteConfig.logo} alt="" width="32" height="32" /><span>{siteConfig.title}</span>
			</a>
			<Sidebar {entries} />
		</aside>
	{/if}
	<dialog id="docs-mobile-menu" class="m-0 h-dvh max-h-dvh w-[min(85vw,360px)] max-w-full border-0 bg-(--docs-panel) px-6 pt-4 pb-12 text-(--docs-text) backdrop:bg-black/45" bind:this={mobileMenu} onclose={() => menuOpen = false} aria-label="Navigation">
		<div class="mb-4 flex items-center justify-between"><strong>Navigation</strong><button class="size-10 text-[28px]" type="button" aria-label="Close navigation" onclick={() => mobileMenu.close()}>×</button></div>
		<nav class="border-b border-(--docs-border) pb-4 [&_a]:block [&_a]:py-2" aria-label="Main navigation">
			{#each navbar as link}<a href={link.to}>{link.title}</a>{/each}
		</nav>
		{#if entries.length}<Sidebar {entries} />{/if}
	</dialog>
	<main class="min-w-0 pt-20 min-[950px]:pt-[105px]" id="docs-main" tabindex="-1">{@render children()}</main>
</div>

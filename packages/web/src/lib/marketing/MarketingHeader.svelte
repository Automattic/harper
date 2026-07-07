<script lang="ts">
import { Arrow, DiscordLogo, GitHubLogo } from 'components';
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

<header
	class={`sticky top-0 z-40 border-b backdrop-blur-md transition duration-200 ease-out motion-reduce:transition-none ${
		compact
			? 'border-black/10 bg-cream/90 shadow-[0_1px_0_rgba(0,0,0,0.04),0_8px_24px_-20px_rgba(0,0,0,0.18)] dark:border-white/10 dark:bg-black/90'
			: 'border-transparent bg-cream/75 dark:bg-black/75'
	}`}
>
	<div
		class={`mx-auto grid max-w-[87.5rem] grid-cols-[1fr_auto_1fr] items-center gap-6 px-10 transition-[padding] duration-200 motion-reduce:transition-none max-[900px]:grid-cols-[1fr_auto] max-[900px]:px-4 ${
			compact ? 'py-1.5' : 'py-[1.15rem]'
		}`}
	>
		<a
			class="inline-flex items-center gap-3 !text-black no-underline transition-[gap] duration-200 motion-reduce:transition-none dark:!text-white [&_.harper-mark]:transition-[width,height] [&_.harper-mark]:duration-200"
			href="/"
			aria-label="Harper home"
		>
			<HarperMark size={compact ? 24 : 30} />
			<strong class="text-lg font-bold tracking-normal transition-[font-size] duration-200 motion-reduce:transition-none">Harper</strong>
		</a>

		<nav
			class="flex items-center gap-3 justify-self-center transition-[gap] duration-200 motion-reduce:transition-none max-[900px]:hidden"
			aria-label="Marketing navigation"
		>
			<a
				class={`rounded-lg px-3 py-2 text-[0.94rem] font-semibold !text-black no-underline transition-[background,padding,font-size] duration-200 hover:bg-black/5 hover:no-underline motion-reduce:transition-none dark:!text-white dark:hover:bg-white/10 ${
					compact ? 'py-1.5' : ''
				} ${active === 'docs' ? 'font-bold' : ''}`}
				href="/docs/about">Documentation</a
			>
			<a
				class={`inline-flex h-[2.125rem] items-center gap-2 rounded-lg border-[0.5px] border-black/25 px-3.5 py-0 !text-black no-underline transition-[background,border-color,color,height,padding,gap,transform] duration-200 hover:translate-y-[-0.5px] hover:no-underline motion-reduce:transition-none dark:border-white/25 dark:!text-white [&_path]:fill-none [&_path]:stroke-current [&_path]:stroke-[1.5] [&_path]:[stroke-linecap:round] [&_path]:[stroke-linejoin:round] [&_svg]:size-3 ${
					compact ? 'h-8 px-3' : ''
				} ${
					ctaPrimary
						? 'border-primary bg-primary !text-white hover:!text-white dark:border-primary-300 dark:bg-primary-300 dark:!text-black dark:hover:!text-black'
						: ''
				} ${active === 'get' ? 'font-bold' : ''}`}
				href="/get"
			>
				<span>Get Harper</span>
				<Arrow />
			</a>
		</nav>

		<div
			class={`flex items-center justify-end gap-1.5 transition-[gap] duration-200 motion-reduce:transition-none [&_.marketing-docsearch]:transition-[min-width] [&_.marketing-docsearch]:duration-200 [&_.marketing-docsearch]:motion-reduce:transition-none [&_.marketing-docsearch_.DocSearch-Button]:transition-[background,color,height,padding,min-width] [&_.marketing-docsearch_.DocSearch-Button]:duration-200 [&_.marketing-docsearch_.DocSearch-Button-Keys]:transition-[width,height,min-width,font-size] [&_.marketing-docsearch_.DocSearch-Button-Keys]:duration-200 [&_.marketing-docsearch_.DocSearch-Button-Placeholder]:transition-[padding,font-size] [&_.marketing-docsearch_.DocSearch-Button-Placeholder]:duration-200 [&_.marketing-docsearch_.DocSearch-Search-Icon]:transition-[width,height,min-width,font-size] [&_.marketing-docsearch_.DocSearch-Search-Icon]:duration-200 max-[900px]:[&_.marketing-docsearch]:hidden ${
				compact
					? '[&_.marketing-docsearch]:min-w-[10.25rem] [&_.marketing-docsearch_.DocSearch-Button]:h-[2.125rem] [&_.marketing-docsearch_.DocSearch-Button]:px-2.5 [&_.marketing-docsearch_.DocSearch-Button-Keys]:h-[1.18rem] [&_.marketing-docsearch_.DocSearch-Button-Keys]:min-w-8 [&_.marketing-docsearch_.DocSearch-Button-Keys]:text-[0.62rem] [&_.marketing-docsearch_.DocSearch-Button-Placeholder]:pr-2 [&_.marketing-docsearch_.DocSearch-Button-Placeholder]:text-[0.92rem] [&_.marketing-docsearch_.DocSearch-Search-Icon]:size-[0.95rem]'
					: ''
			}`}
		>
			<a
				class="inline-flex size-8 items-center justify-center rounded-full !text-black/70 no-underline transition-[background,color,width,height] duration-200 hover:bg-black/5 hover:!text-black hover:no-underline motion-reduce:transition-none dark:!text-white/70 dark:hover:bg-white/10 dark:hover:!text-white max-[900px]:hidden [&_svg]:size-4 [&_svg]:fill-current [&_svg]:transition-[width,height] [&_svg]:duration-200"
				href={marketingLinks.github}
				aria-label="GitHub"
			>
				<GitHubLogo />
			</a>
			<a
				class="inline-flex size-8 items-center justify-center rounded-full !text-black/70 no-underline transition-[background,color,width,height] duration-200 hover:bg-black/5 hover:!text-black hover:no-underline motion-reduce:transition-none dark:!text-white/70 dark:hover:bg-white/10 dark:hover:!text-white max-[900px]:hidden [&_svg]:size-4 [&_svg]:fill-current [&_svg]:transition-[width,height] [&_svg]:duration-200"
				href={marketingLinks.discord}
				aria-label="Discord"
			>
				<DiscordLogo />
			</a>
			<MarketingDocSearch />
			<button
				class="hidden size-8 flex-col items-center justify-center gap-1 rounded-full border-0 bg-transparent text-black transition-[width,height,gap] duration-200 motion-reduce:transition-none dark:text-white max-[900px]:inline-flex [&_span]:h-[1.5px] [&_span]:w-4 [&_span]:rounded-full [&_span]:bg-current [&_span]:transition-[width,height] [&_span]:duration-200"
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
		<div class="hidden gap-1 px-4 pb-4 max-[900px]:grid">
			<a
				class="rounded-xl bg-white/65 px-4 py-3 font-bold !text-black no-underline hover:no-underline dark:bg-white/10 dark:!text-white"
				href="/docs/about"
				on:click={() => (mobileOpen = false)}>Documentation</a
			>
			<a
				class="rounded-xl bg-white/65 px-4 py-3 font-bold !text-black no-underline hover:no-underline dark:bg-white/10 dark:!text-white"
				href="/get"
				on:click={() => (mobileOpen = false)}>Get Harper</a
			>
			<a
				class="rounded-xl bg-white/65 px-4 py-3 font-bold !text-black no-underline hover:no-underline dark:bg-white/10 dark:!text-white"
				href="/desktop"
				on:click={() => (mobileOpen = false)}>Harper Desktop</a
			>
			<a
				class="rounded-xl bg-white/65 px-4 py-3 font-bold !text-black no-underline hover:no-underline dark:bg-white/10 dark:!text-white"
				href={marketingLinks.github}
				on:click={() => (mobileOpen = false)}>GitHub</a
			>
			<a
				class="rounded-xl bg-white/65 px-4 py-3 font-bold !text-black no-underline hover:no-underline dark:bg-white/10 dark:!text-white"
				href={marketingLinks.discord}
				on:click={() => (mobileOpen = false)}>Discord</a
			>
		</div>
	{/if}
</header>

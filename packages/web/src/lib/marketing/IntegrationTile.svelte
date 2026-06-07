<script lang="ts">
import ChromeLogo from '$lib/components/ChromeLogo.svelte';
import CodeLogo from '$lib/components/CodeLogo.svelte';
import EmacsLogo from '$lib/components/EmacsLogo.svelte';
import FirefoxLogo from '$lib/components/FirefoxLogo.svelte';
import HelixLogo from '$lib/components/HelixLogo.svelte';
import NeovimLogo from '$lib/components/NeovimLogo.svelte';
import ObsidianLogo from '$lib/components/ObsidianLogo.svelte';
import SublimeLogo from '$lib/components/SublimeLogo.svelte';
import WordPressLogo from '$lib/components/WordPressLogo.svelte';
import ZedLogo from '$lib/components/ZedLogo.svelte';
import type { Integration } from './data';
import HarperMark from './HarperMark.svelte';

export let integration: Integration;
export let size = 40;

const baseTileClasses =
	'inline-flex shrink-0 items-center justify-center shadow-[0_0_0_0.5px_rgba(0,0,0,0.08),0_1px_2px_rgba(0,0,0,0.04)]';
const desktopTileClasses = `${baseTileClasses} border border-black bg-white text-black shadow-none dark:border-white dark:bg-black dark:text-white`;
const logoTileClasses = `${baseTileClasses} bg-white dark:bg-white [&_svg]:h-[68%] [&_svg]:w-[68%]`;
const monogramTileClasses = `${baseTileClasses} bg-[var(--tile-bg)] text-[length:var(--tile-font)] font-extrabold tracking-normal text-[var(--tile-fg)]`;

$: tileStyle = `width: ${size}px; height: ${size}px; border-radius: ${Math.round(size * 0.235)}px;`;
$: markSize = Math.round(size * 0.55);
$: fontSize =
	integration.initial && integration.initial.length > 1
		? Math.round(size * 0.36)
		: Math.round(size * 0.48);
</script>

{#if integration.id === 'desktop'}
	<span class={desktopTileClasses} style={tileStyle} aria-hidden="true">
		<HarperMark size={markSize} title="" />
	</span>
{:else if integration.id === 'chrome'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<ChromeLogo />
	</span>
{:else if integration.id === 'firefox'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<FirefoxLogo />
	</span>
{:else if integration.id === 'vscode'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<CodeLogo />
	</span>
{:else if integration.id === 'neovim'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<NeovimLogo />
	</span>
{:else if integration.id === 'wordpress'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<WordPressLogo />
	</span>
{:else if integration.id === 'obsidian'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<ObsidianLogo />
	</span>
{:else if integration.id === 'zed'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<ZedLogo />
	</span>
{:else if integration.id === 'helix'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<HelixLogo />
	</span>
{:else if integration.id === 'emacs'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<EmacsLogo />
	</span>
{:else if integration.id === 'sublime'}
	<span class={logoTileClasses} style={tileStyle} aria-hidden="true">
		<SublimeLogo />
	</span>
{:else}
	<span
		class={monogramTileClasses}
		style={`${tileStyle} --tile-bg: ${integration.color ?? '#1c1a16'}; --tile-fg: ${integration.fg ?? '#fff'}; --tile-font: ${fontSize}px;`}
		aria-hidden="true"
	>
		{integration.initial ?? integration.name[0]}
	</span>
{/if}

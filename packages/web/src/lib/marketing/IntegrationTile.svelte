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

$: tileStyle = `width: ${size}px; height: ${size}px; border-radius: ${Math.round(size * 0.235)}px;`;
$: markSize = Math.round(size * 0.55);
$: fontSize =
	integration.initial && integration.initial.length > 1
		? Math.round(size * 0.36)
		: Math.round(size * 0.48);
</script>

{#if integration.id === 'desktop'}
	<span class="tile desktop" style={tileStyle} aria-hidden="true">
		<HarperMark size={markSize} title="" />
	</span>
{:else if integration.id === 'chrome'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<ChromeLogo />
	</span>
{:else if integration.id === 'firefox'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<FirefoxLogo />
	</span>
{:else if integration.id === 'vscode'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<CodeLogo />
	</span>
{:else if integration.id === 'neovim'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<NeovimLogo />
	</span>
{:else if integration.id === 'wordpress'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<WordPressLogo />
	</span>
{:else if integration.id === 'obsidian'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<ObsidianLogo />
	</span>
{:else if integration.id === 'zed'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<ZedLogo />
	</span>
{:else if integration.id === 'helix'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<HelixLogo />
	</span>
{:else if integration.id === 'emacs'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<EmacsLogo />
	</span>
{:else if integration.id === 'sublime'}
	<span class="tile logo component-logo" style={tileStyle} aria-hidden="true">
		<SublimeLogo />
	</span>
{:else}
	<span
		class="tile monogram"
		style={`${tileStyle} --tile-bg: ${integration.color ?? '#1c1a16'}; --tile-fg: ${integration.fg ?? '#fff'}; --tile-font: ${fontSize}px;`}
		aria-hidden="true"
	>
		{integration.initial ?? integration.name[0]}
	</span>
{/if}

<style>
.tile {
	display: inline-flex;
	flex-shrink: 0;
	align-items: center;
	justify-content: center;
	box-shadow:
		0 0 0 0.5px rgba(28, 26, 22, 0.08),
		0 1px 2px rgba(28, 26, 22, 0.04);
}

.desktop {
	border: 1px solid var(--marketing-ink);
	background: #fff;
	color: var(--marketing-ink);
	box-shadow: none;
}

.logo {
	background: #fff;
}

.component-logo :global(svg) {
	display: block;
	width: 68%;
	height: 68%;
}

.monogram {
	background: var(--tile-bg);
	color: var(--tile-fg);
	font-family: inherit;
	font-size: var(--tile-font);
	font-weight: 800;
	letter-spacing: 0;
}
</style>

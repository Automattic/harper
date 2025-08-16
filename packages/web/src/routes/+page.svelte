<script module>
import ChromeLogo from '$lib/ChromeLogo.svelte';
import CodeLogo from '$lib/CodeLogo.svelte';
import Editor from '$lib/Editor.svelte';
import FirefoxLogo from '$lib/FirefoxLogo.svelte';
import GitHubLogo from '$lib/GitHubLogo.svelte';
import Graph from '$lib/Graph.svelte';
import Logo from '$lib/Logo.svelte';
import ObsidianLogo from '$lib/ObsidianLogo.svelte';
import Section from '$lib/Section.svelte';

export const frontmatter = {
	home: false,
};

let width = $state(window.innerWidth);

window.addEventListener('resize', () => {
	width = window.innerWidth;
});

let mobile = $derived(width < 640);

/**
 * @param {string} keyword
 */
function agentHas(keyword) {
	return navigator.userAgent.toLowerCase().search(keyword.toLowerCase()) > -1;
}
</script>

<div class="w-full flex flex-col items-center">
	<Logo width="200px" />
</div>
<h1 class="font-bold text-center">Hi. I’m Harper.</h1>
<h2 class="text-center text-lg md:text-2xl lg:text-4xl">The Grammar Checker That Respects Your Privacy</h2>

<div
	class="md:flex md:flex-row grid grid-cols-2 items-center justify-evenly mt-5 transition-all place-items-center"
>
	<a
		href="https://github.com/automattic/harper"
		class="flex flex-row items-center [&>*]:m-2 hover:scale-105"
		><GitHubLogo width="40px" height="40px" />GitHub</a
	>

  {#if agentHas("firefox")}
	  <a href="https://addons.mozilla.org/en-US/firefox/addon/private-grammar-checker-harper/" class="flex flex-row items-center [&>*]:m-2 hover:scale-105"
	  	><FirefoxLogo width="40px" height="40px" />Firefox Extension</a
	  >
  {:else}
	  <a href="https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb" class="flex flex-row items-center [&>*]:m-2 hover:scale-105"
	  	><ChromeLogo width="40px" height="40px" />Chrome Extension</a
	  >
  {/if}

	<a href="/docs/integrations/obsidian" class="flex flex-row items-center [&>*]:m-2 hover:scale-105"
		><ObsidianLogo width="40px" height="40px" />Obsidian Plugin</a
	>

	<a
		href="https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper"
		class="flex flex-row items-center [&>*]:m-2 hover:scale-105"
		><CodeLogo width="40px" height="40px" />Code Plugin</a
	>
	<a href="https://elijahpotter.dev" class="flex flex-row items-center [&>*]:m-2 hover:scale-105"
		><img
			width="40"
			height="40"
			class="hover:scale-105 transition-all"
			src="/icons/profile.svg"
			alt="Author"
		/>Author</a
	>
</div>

<Section noChild>
	<span slot="title"> What is it? </span>
	<span slot="subtitle">
		Harper is a free English grammar checker designed to be <em>just right</em>. You can think of it as
		an open-source alternative to Grammarly. I created it after years of dealing with the
		shortcomings of the competition.
	</span>
</Section>

<div class="w-full h-[800px] overflow-hidden">
	<Editor />
</div>

<Section>
	<span slot="title">Private</span>
	<span slot="subtitle"
		>Harper is completely private, in every sense of the word. <br /><br />
		Since Harper runs on-device, your data doesn't go anywhere you don't want it to. <br />
		<br />That means you have 100% certainty we don't violate your copyright by training large
		language models.
	</span>

	<img src="/images/camera.webp" class="rounded" alt="Graffiti of a camera." />
</Section>

<Section swapped={!mobile}>
	<span slot="title">Native Everywhere</span>
	<span slot="subtitle"
		>Harper is available as a <a
			href="/docs/integrations/language-server">language server</a
		>, <a href="/docs/harperjs/introduction">JavaScript library</a
		> through WebAssembly, and <a
		href="https://crates.io/crates/harper-core">Rust crate</a
		>, so you can get fantastic grammar checking anywhere you work.
		<br /><br /> That said, we take extra care to make sure the
		<a href="/docs/integrations/visual-studio-code"
			>Visual Studio Code</a
		>, <a href="/docs/integrations/neovim">Neovim</a>,
		<a href="/docs/integrations/obsidian">Obsidian</a>, and <a href="/docs/integrations/chrome-extension">Chrome</a> integrations are amazing.
	</span>

	<img
		src={['/images/harper_wp_playground_screenshot.png', '/images/obsidian_screenshot.webp'][
			Math.floor(Math.random() * 2)
		]}
		class="dark:invert rounded"
		alt="A screenshot of a text editor with Harper suggestions."
	/></Section
>

<Section>
	<span slot="title">Wicked Fast</span>
	<span slot="subtitle"
		>Since Harper runs on <strong>your</strong> devices, it's able to serve up suggestions in under
		10 milliseconds.
		<br />
		<br />
		No network request, no massive language models, no fuss.</span
	>
	<Graph /></Section
>

<Section noChild swapped={!mobile}>
	<span slot="title">Open Source</span>
	<span slot="subtitle"
		>Harper is completely open source under the Apache-2.0 license. <br /><br /> Come pay us a visit
		on <a href="https://github.com/automattic/harper">GitHub.</a>
	</span>
</Section>

<script lang="ts">
import { marketingLinks } from './data';
import HarperMark from './HarperMark.svelte';
</script>

<footer class="border-t border-black/10 bg-cream text-black/50 dark:border-white/10 dark:bg-black dark:text-white/50">
	<div class="mx-auto grid max-w-[77.5rem] grid-cols-[1.4fr_1fr_1fr_1fr] gap-10 px-10 pt-12 pb-7 max-[820px]:grid-cols-2 max-[820px]:px-4 max-[560px]:grid-cols-1">
		<div>
			<div class="flex items-center gap-2.5 text-black dark:text-white">
				<HarperMark size={26} />
				<strong class="text-sm">Harper</strong>
			</div>
			<p class="!mt-3.5 !mb-4 max-w-72 text-[0.84rem] leading-[1.55] text-black/70 dark:text-white/70">
				The free, private, open-source grammar checker that lives on your device.
			</p>
			<div
				class='inline-flex items-center gap-2 rounded-full border border-black/10 bg-white px-2.5 py-1.5 text-[0.68rem] text-black/70 dark:border-white/10 dark:bg-white/10 dark:text-white/70 [font-family:"JetBrains_Mono",monospace]'
			>
				<span class="size-2 rounded-full bg-green-700"></span>Apache-2.0 · v1.0
			</div>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class='!m-0 py-0 text-[0.66rem] font-extrabold tracking-[0.14em] text-primary uppercase dark:text-primary-300 [font-family:"JetBrains_Mono",monospace]'>
				Get Harper
			</h2>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/desktop">Harper Desktop</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href={marketingLinks.chrome}>Chrome extension</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href={marketingLinks.vscode}>VS Code</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/integrations/obsidian">Obsidian plugin</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/get">All integrations</a>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class='!m-0 py-0 text-[0.66rem] font-extrabold tracking-[0.14em] text-primary uppercase dark:text-primary-300 [font-family:"JetBrains_Mono",monospace]'>
				Resources
			</h2>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/about">Documentation</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/rules">Rule reference</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/integrations/language-server">harper-ls</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/harperjs/introduction">harper.js</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href={marketingLinks.crates}>harper-core</a>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class='!m-0 py-0 text-[0.66rem] font-extrabold tracking-[0.14em] text-primary uppercase dark:text-primary-300 [font-family:"JetBrains_Mono",monospace]'>
				Community
			</h2>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href={marketingLinks.github}>GitHub</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href={marketingLinks.discord}>Discord</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/contributors/introduction">Contributing</a>
		</div>
	</div>

	<div class="mx-auto flex max-w-[77.5rem] items-center justify-between gap-4 border-t border-black/10 px-10 py-4 text-[0.78rem] dark:border-white/10 max-[820px]:flex-col max-[820px]:items-start max-[820px]:px-4">
		<span>An <a class="!text-black/70 hover:!text-black dark:!text-white/70 dark:hover:!text-white" href="https://automattic.com/">Automattic</a> open-source project.</span>
		<div class="flex gap-4">
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href="/docs/about">Privacy</a>
			<a class="!text-black/70 no-underline hover:!text-black hover:no-underline dark:!text-white/70 dark:hover:!text-white" href={marketingLinks.github}>License</a>
		</div>
	</div>
</footer>

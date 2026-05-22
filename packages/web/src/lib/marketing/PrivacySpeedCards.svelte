<script lang="ts">
import Graph from '$lib/components/Graph.svelte';

export let desktop = false;
</script>

<div class="grid grid-cols-1 gap-7 md:grid-cols-2">
	<article
		class="flex min-h-92 flex-col gap-4 overflow-hidden rounded-2xl border border-black/10 bg-white p-6 text-black dark:border-white/10 dark:bg-black dark:text-white md:p-8"
	>
		<h2 class="m-0 py-0 text-[clamp(2rem,4vw,2.25rem)] leading-[1.08] font-semibold tracking-normal">
			{desktop ? 'Your words never leave your device.' : 'Harper is completely private.'}
		</h2>
		<p class="!m-0 text-sm leading-relaxed text-black/70 dark:text-white/70">
			{#if desktop}
				Like the rest of Harper, Harper Desktop runs entirely on your Mac. No cloud round-trips,
				no telemetry, no account, no LLM in the loop.
			{:else}
				Every check happens locally. No cloud round-trips, no telemetry, no LLM in the loop.
				That means certainty that we never train models on your writing.
			{/if}
		</p>
		<div
			class='mt-auto flex flex-wrap items-center gap-3 text-xs text-black/50 dark:text-white/50 [font-family:"JetBrains_Mono",monospace] md:flex-nowrap'
			aria-label="Your writing stays on your device"
		>
			<span
				class="inline-flex whitespace-nowrap rounded-full border border-primary bg-primary-100 px-3.5 py-2 text-black"
			>
				Your {desktop ? 'Mac' : 'device'}
			</span>
			<svg
				class="h-6 w-16 shrink-0 fill-none stroke-current stroke-[1.4] [stroke-linecap:round] [stroke-linejoin:round]"
				viewBox="0 0 62 22"
				aria-hidden="true"
			>
				<path d="M2 11 H22" />
				<path d="M40 11 H54" />
				<path d="M50 6 L56 11 L50 16" />
				<circle class="fill-white stroke-primary stroke-[1.6] dark:fill-black" cx="31" cy="11" r="9" />
				<path class="stroke-primary stroke-[1.6]" d="M25 17 L37 5" />
			</svg>
			<span
				class="inline-flex whitespace-nowrap rounded-full border border-black/10 bg-transparent px-3.5 py-2 text-black/70 line-through dark:border-white/15 dark:text-white/70"
			>
				The cloud
			</span>
		</div>
	</article>

	<article
		class="flex min-h-92 flex-col gap-4 overflow-hidden rounded-2xl bg-black p-6 text-white dark:bg-white dark:text-black md:p-8"
	>
		<h2 class="m-0 py-0 text-[clamp(2rem,4vw,2.25rem)] leading-[1.08] font-semibold tracking-normal">
			Suggestions in under 10ms.
		</h2>
		<p class="!m-0 text-sm leading-relaxed text-white/70 dark:text-black/70">
			Harper runs locally and is built for speed. You get a feedback loop that keeps up with
			your typing, without waiting for a server.
		</p>
		<Graph />
	</article>
</div>

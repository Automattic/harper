<script lang="ts">
import SectionKicker from './SectionKicker.svelte';

export let desktop = false;
</script>

<div class="cards">
	<article class="card privacy">
		<SectionKicker>{desktop ? 'Private' : 'Private by design'}</SectionKicker>
		<h2>{desktop ? 'Your words never leave your device.' : 'Harper is completely private.'}</h2>
		<p>
			{#if desktop}
				Like the rest of Harper, Harper Desktop runs entirely on your Mac. No cloud round-trips,
				no telemetry, no account, no LLM in the loop.
			{:else}
				Every check happens locally. No cloud round-trips, no telemetry, no LLM in the loop.
				That means certainty that we never train models on your writing.
			{/if}
		</p>
		<div class="privacy-viz" aria-label="Your writing stays on your device">
			<span>Your {desktop ? 'Mac' : 'device'}</span>
			<svg viewBox="0 0 62 22" aria-hidden="true">
				<path d="M2 11 H22" />
				<path d="M40 11 H54" />
				<path d="M50 6 L56 11 L50 16" />
				<circle cx="31" cy="11" r="9" />
				<path class="slash" d="M25 17 L37 5" />
			</svg>
			<span class="dim">The cloud</span>
		</div>
	</article>

	<article class="card speed">
		<SectionKicker color="var(--marketing-amber-soft)">Wicked fast</SectionKicker>
		<h2>Suggestions in under 10ms.</h2>
		<p>
			Harper runs locally and is built for speed. You get a feedback loop that keeps up with
			your typing, without waiting for a server.
		</p>
		<div class="latency">
			<div>
				<span>Harper</span>
				<b><i style="width: 3%"></i></b>
				<em>8 ms</em>
			</div>
			<div>
				<span>Cloud checker</span>
				<b><i style="width: 88%"></i></b>
				<em>280 ms</em>
			</div>
		</div>
	</article>
</div>

<style>
.cards {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 1.75rem;
}

.card {
	display: flex;
	flex-direction: column;
	gap: 1.1rem;
	min-height: 23rem;
	padding: 2rem;
	border-radius: 1rem;
	overflow: hidden;
}

.privacy {
	border: 0.5px solid var(--marketing-line);
	background: #fff;
}

.speed {
	background: var(--marketing-ink);
	color: #fbfaf6;
}

h2 {
	margin: 0;
	color: inherit;
	font-family: var(--marketing-display);
	font-size: clamp(2rem, 4vw, 2.25rem);
	font-weight: 650;
	line-height: 1.08;
	letter-spacing: 0;
}

p {
	margin: 0;
	color: var(--marketing-ink-2);
	font-family: var(--marketing-sans);
	font-size: 0.94rem;
	line-height: 1.6;
}

.speed p {
	color: rgba(251, 250, 246, 0.72);
}

.privacy-viz {
	display: flex;
	align-items: center;
	gap: 0.85rem;
	margin-top: auto;
	color: var(--marketing-ink-3);
	font-family: var(--marketing-mono);
	font-size: 0.7rem;
}

.privacy-viz span {
	display: inline-flex;
	padding: 0.5rem 0.85rem;
	border: 0.5px solid var(--marketing-amber);
	border-radius: 999px;
	background: var(--marketing-amber-tint);
	color: var(--marketing-ink);
	white-space: nowrap;
}

.privacy-viz .dim {
	border-color: var(--marketing-line);
	background: transparent;
	color: var(--marketing-ink-2);
	text-decoration: line-through;
}

.privacy-viz svg {
	width: 3.9rem;
	height: 1.4rem;
	flex-shrink: 0;
	fill: none;
	stroke: var(--marketing-ink-3);
	stroke-linecap: round;
	stroke-linejoin: round;
	stroke-width: 1.4;
}

.privacy-viz circle {
	fill: #fff;
	stroke: var(--marketing-amber);
	stroke-width: 1.6;
}

.privacy-viz .slash {
	stroke: var(--marketing-amber);
	stroke-width: 1.6;
}

.latency {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	margin-top: auto;
}

.latency div {
	display: grid;
	grid-template-columns: 6rem 1fr 3.5rem;
	align-items: center;
	gap: 0.65rem;
	font-family: var(--marketing-mono);
	font-size: 0.7rem;
}

.latency b {
	height: 0.62rem;
	border-radius: 999px;
	background: rgba(251, 250, 246, 0.08);
	overflow: hidden;
}

.latency i {
	display: block;
	height: 100%;
	border-radius: inherit;
	background: var(--marketing-amber-soft);
}

.latency div + div i {
	background: rgba(251, 250, 246, 0.18);
}

.latency em {
	color: rgba(251, 250, 246, 0.75);
	font-style: normal;
	text-align: right;
}

@media (max-width: 760px) {
	.cards {
		grid-template-columns: 1fr;
	}

	.card {
		min-height: auto;
		padding: 1.4rem;
	}

	.privacy-viz {
		flex-wrap: wrap;
	}
}
</style>

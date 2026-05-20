<script lang="ts">
import SectionKicker from './SectionKicker.svelte';

type FaqItem = {
	q: string;
	a: string;
};

export let items: FaqItem[] = [];
export let title = 'FAQs';
export let kicker = '';
export let intro = '';
export let introHref = '';
export let introLinkText = '';
export let collapsible = false;
export let layout: 'grid' | 'narrow' = 'narrow';
</script>

<section class="faq">
	<div class:grid={layout === 'grid'} class:narrow={layout === 'narrow'} class="section-inner">
		<div class="heading">
			{#if kicker}
				<SectionKicker>{kicker}</SectionKicker>
			{/if}
			<h2>{title}</h2>
			{#if intro}
				<p>
					{intro}
					{#if introHref && introLinkText}
						<a href={introHref}>{introLinkText}</a>
					{/if}
				</p>
			{/if}
		</div>

		<div class:faq-list={collapsible} class:faq-rows={!collapsible}>
			{#if collapsible}
				{#each items as item}
					<details>
						<summary>{item.q}</summary>
						<p>{item.a}</p>
					</details>
				{/each}
			{:else}
				{#each items as item}
					<div class="faq-row">
						<h3>{item.q}</h3>
						<p>{item.a}</p>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>

<style>
.faq {
	border-top: 0.5px solid var(--marketing-line);
	background: #fdfbf5;
	padding: 4.8rem 0;
}

.section-inner {
	max-width: 68.75rem;
	margin: 0 auto;
	padding: 0 3.5rem;
}

.section-inner.grid {
	display: grid;
	grid-template-columns: 18.75rem 1fr;
	gap: 3.1rem;
}

.section-inner.narrow {
	max-width: 45rem;
	text-align: center;
}

h2 {
	margin: 0.75rem 0 0;
	color: inherit;
	font-family: var(--marketing-display);
	font-size: clamp(2.2rem, 5vw, 2.5rem);
	font-weight: 650;
	line-height: 1.08;
	letter-spacing: 0;
}

p {
	color: var(--marketing-ink-2);
	font-size: 1rem;
	line-height: 1.65;
}

.heading p {
	margin: 1rem 0 0;
}

a {
	color: var(--marketing-amber);
	font-weight: 700;
	text-decoration: none;
}

.faq-list {
	border-top: 0.5px solid var(--marketing-line);
}

details {
	border-bottom: 0.5px solid var(--marketing-line);
}

summary {
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 1.1rem 0;
	color: var(--marketing-ink);
	cursor: pointer;
	font-weight: 750;
	list-style: none;
}

summary::-webkit-details-marker {
	display: none;
}

summary::after {
	content: '+';
	display: inline-flex;
	width: 1.4rem;
	height: 1.4rem;
	align-items: center;
	justify-content: center;
	border-radius: 999px;
	background: rgba(28, 26, 22, 0.06);
	color: var(--marketing-ink-2);
}

details[open] summary::after {
	content: '-';
	background: var(--marketing-ink);
	color: var(--marketing-amber-tint);
}

details p {
	max-width: 38rem;
	margin: 0;
	padding: 0 0 1.25rem;
}

.faq-row {
	border-top: 0.5px solid var(--marketing-line);
	padding: 1.25rem 0;
	text-align: left;
}

.faq-row:last-child {
	border-bottom: 0.5px solid var(--marketing-line);
}

.faq-row h3 {
	margin: 0 0 0.4rem;
	color: inherit;
	font-family: var(--marketing-display);
	font-size: 1rem;
	font-weight: 650;
	letter-spacing: 0;
}

.faq-row p {
	margin: 0;
}

@media (max-width: 880px) {
	.section-inner {
		padding-inline: 1rem;
	}

	.section-inner.grid {
		grid-template-columns: 1fr;
	}
}
</style>

<script lang="ts">
type FaqItem = {
	q: string;
	a: string;
};

export let items: FaqItem[] = [];
export let title = 'FAQs';
</script>

<section class="faq">
	<div class="section-inner">
		<h2>{title}</h2>
		<div class="faq-rows">
			{#each items as item}
				<div class="faq-row">
					<h3>{item.q}</h3>
					<p>{item.a}</p>
				</div>
			{/each}
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
	max-width: 45rem;
	margin: 0 auto;
	padding: 0 3.5rem;
	text-align: center;
}

h2 {
	margin: 0.75rem 0 0;
	color: inherit;
	font-family: Domine, serif;
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

.faq-row {
	border-top: 0.5px solid var(--marketing-line);
	padding: 1.25rem 0;
	text-align: left;
}

.faq-row:first-child {
	margin-top: 2rem;
}

.faq-row:last-child {
	border-bottom: 0.5px solid var(--marketing-line);
}

.faq-row h3 {
	margin: 0 0 0.4rem;
	color: inherit;
	font-family: Domine, serif;
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
}
</style>

<script lang="ts">
import Testimonial from './Testimonial.svelte';

type TestimonialItem = {
	authorName: string;
	authorSubtitle: string;
	testimonial: string;
	/** The URL the testimonial was sourced from. */
	source: string;
};

export let testimonials: TestimonialItem[] = [];

const { class: extraClass = '', ...restProps } = $$restProps;
</script>

<section {...restProps} class={extraClass}>
	<div class="testimonial-grid">
		{#each testimonials as item, index (item.authorName + index)}
			<a class="testimonial-link" href={item.source}>
				<Testimonial
					authorName={item.authorName}
					authorSubtitle={item.authorSubtitle}
					testimonial={item.testimonial}
					accent={index === 4}
				/>
			</a>
		{/each}
	</div>
</section>

<style>
.testimonial-grid {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 1.1rem;
}

.testimonial-link {
	color: inherit;
	text-decoration: none;
}

.testimonial-link:hover :global(figure) {
	border-color: var(--marketing-amber);
	box-shadow: 0 10px 24px -16px rgba(28, 26, 22, 0.16);
	transform: translateY(-1px);
}

@media (max-width: 880px) {
	.testimonial-grid {
		grid-template-columns: 1fr;
	}
}
</style>

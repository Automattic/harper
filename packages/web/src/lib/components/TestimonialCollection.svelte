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
	<div class="grid grid-cols-1 gap-4 min-[881px]:grid-cols-3">
		{#each testimonials as item, index (item.authorName + index)}
			<a class="group text-inherit no-underline hover:no-underline" href={item.source}>
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

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

<section
	{...restProps}
	class={`mx-auto w-full max-w-6xl ${extraClass}`.trim()}
>
	<div class="columns-1 gap-6 sm:columns-2 lg:columns-3">
		{#each testimonials as item, index (item.authorName + index)}
			<a href={item.source} class={`block mb-6 break-inside-avoid no-underline ${index % 2 == 0 ? "skew-hover" : "skew-hover-left"}`}>
				<Testimonial
					authorName={item.authorName}
					authorSubtitle={item.authorSubtitle}
					testimonial={item.testimonial}
				/>
			</a>
		{/each}
	</div>
</section>

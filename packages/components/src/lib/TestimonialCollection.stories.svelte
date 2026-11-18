<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import TestimonialCollection from './TestimonialCollection.svelte';

const { Story } = defineMeta({
	title: 'Components/TestimonialCollection',
	component: TestimonialCollection,
	tags: ['autodocs'],
	parameters: {
		layout: 'padded',
	},
	args: {
		testimonials: [
			{
				authorName: 'Alex Morgan',
				authorSubtitle: 'Example author',
				testimonial: 'This is the first example testimonial.',
				source: 'https://example.com/testimonials/1',
			},
			{
				authorName: 'Sam Taylor',
				authorSubtitle: 'Example author',
				testimonial: 'A longer example shows how the card handles text across multiple lines.',
				source: 'https://example.com/testimonials/2',
			},
			{
				authorName: 'Jordan Lee',
				authorSubtitle: 'Example author',
				testimonial: 'This is the third example testimonial.',
				source: 'https://example.com/testimonials/3',
			},
		],
	},
});
</script>

<Story name="Default" />

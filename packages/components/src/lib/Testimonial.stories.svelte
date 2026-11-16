<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Testimonial from './Testimonial.svelte';

const { Story } = defineMeta({
	title: 'Components/Testimonial',
	component: Testimonial,
	tags: ['autodocs'],
	args: {
		authorName: 'Alex Morgan',
		authorSubtitle: 'Example author',
		testimonial: 'This is an example testimonial for the component preview.',
		accent: false,
	},
	argTypes: {
		accent: { control: 'boolean' },
	},
});
</script>

<Story name="Default" />

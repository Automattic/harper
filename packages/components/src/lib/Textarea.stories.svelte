<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Textarea from './Textarea.svelte';

const { Story } = defineMeta({
	title: 'Components/Textarea',
	component: Textarea,
	tags: ['autodocs'],
	args: {
		value: '',
		placeholder: 'Enter a message',
		rows: 4,
		cols: 40,
		'aria-label': 'Message',
	},
});
</script>

<Story name="Default" />

<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Select from './Select.svelte';

const { Story } = defineMeta({
	title: 'Components/Select',
	component: Select,
	tags: ['autodocs'],
	args: {
		items: [
			{ value: 'apple', name: 'Apple' },
			{ value: 'banana', name: 'Banana' },
			{ value: 'orange', name: 'Orange' },
		],
		value: 'apple',
		size: 'md',
		unstyled: false,
		'aria-label': 'Fruit',
	},
	argTypes: {
		size: {
			control: 'select',
			options: ['sm', 'md', 'lg'],
		},
		unstyled: { control: 'boolean' },
	},
});
</script>

<Story name="Default" />

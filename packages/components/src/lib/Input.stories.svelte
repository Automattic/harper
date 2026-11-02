<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Input from './Input.svelte';

const { Story } = defineMeta({
	title: 'Components/Input',
	component: Input,
	tags: ['autodocs'],
	args: {
		type: 'text',
		value: '',
		placeholder: 'Enter a name',
		size: 'md',
		unstyled: false,
		'aria-label': 'Name',
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

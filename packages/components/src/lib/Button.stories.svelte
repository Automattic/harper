<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Button from './Button.svelte';

const { Story } = defineMeta({
	title: 'Components/Button',
	component: Button,
	tags: ['autodocs'],
	args: {
		color: 'primary',
		size: 'md',
		pill: false,
		state: 'idle',
		disabled: false,
		unstyled: false,
	},
	argTypes: {
		color: {
			control: 'select',
			options: ['primary', 'gray', 'white', 'dark'],
		},
		size: {
			control: 'select',
			options: ['xs', 'sm', 'md', 'lg'],
		},
		state: {
			control: 'select',
			options: ['idle', 'loading', 'success'],
		},
		pill: { control: 'boolean' },
		disabled: { control: 'boolean' },
		unstyled: { control: 'boolean' },
		children: { control: false },
	},
});
</script>

<Story name="Default">Click me</Story>

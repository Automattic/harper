<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Toggle from './Toggle.svelte';

const { Story } = defineMeta({
	title: 'Components/Toggle',
	component: Toggle,
	tags: ['autodocs'],
	args: {
		checked: false,
		disabled: false,
		appearance: 'default',
		'aria-label': 'Enable notifications',
	},
	argTypes: {
		checked: { control: 'boolean' },
		disabled: { control: 'boolean' },
		appearance: { control: false },
	},
});
</script>

<script lang="ts">
let settingsChecked = $state(false);
</script>

<Story name="Default">
	Enable notifications
</Story>

<Story
	name="Settings"
	args={{ appearance: 'settings' }}
	argTypes={{ checked: { control: false } }}
>
	{#snippet template(args)}
		<Toggle {...args} checked={settingsChecked} on:click={() => (settingsChecked = !settingsChecked)} />
	{/snippet}
</Story>

<Story name="Disabled" args={{ disabled: true }}>
	Enable notifications
</Story>

<Story name="Disabled Checked" args={{ disabled: true, checked: true }}>
	Enable notifications
</Story>

<Story name="Settings Disabled" args={{ appearance: 'settings', disabled: true }} />

<Story
	name="Settings Disabled Checked"
	args={{ appearance: 'settings', disabled: true, checked: true }}
/>

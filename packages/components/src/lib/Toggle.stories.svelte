<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import { expect, userEvent, within } from 'storybook/test';
import Toggle from './Toggle.svelte';

async function checkDisabledToggle({ canvasElement }: { canvasElement: HTMLElement }) {
	const toggle = within(canvasElement).getByRole('switch', { name: 'Enable notifications' });
	const track = toggle instanceof HTMLInputElement ? toggle.nextElementSibling : toggle;
	const wasChecked =
		toggle instanceof HTMLInputElement
			? toggle.checked
			: toggle.getAttribute('aria-checked') === 'true';

	await expect(toggle).toBeDisabled();
	await expect(track).toHaveStyle({ opacity: '0.5', cursor: 'not-allowed' });
	await expect(getComputedStyle(track as Element).filter).toContain('grayscale');
	await userEvent.click(toggle);
	if (wasChecked) await expect(toggle).toBeChecked();
	else await expect(toggle).not.toBeChecked();
}

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

<Story
	name="Default"
	play={async ({ canvasElement }) => {
		const toggle = within(canvasElement).getByRole('switch', { name: 'Enable notifications' });
		await expect(toggle).not.toBeChecked();
		await expect(toggle.nextElementSibling).toHaveStyle({ width: '44px', height: '24px' });
		await userEvent.click(toggle);
		await expect(toggle).toBeChecked();
		await expect(toggle.nextElementSibling).toHaveStyle({ backgroundColor: 'rgb(193, 117, 11)' });
		await userEvent.keyboard(' ');
		await expect(toggle).not.toBeChecked();
	}}
>
	Enable notifications
</Story>

<Story
	name="Settings"
	args={{ appearance: 'settings' }}
	argTypes={{ checked: { control: false } }}
	play={async ({ canvasElement }) => {
		const toggle = within(canvasElement).getByRole('switch', { name: 'Enable notifications' });
		await expect(toggle).toHaveAttribute('aria-checked', 'false');
		await expect(toggle).toHaveStyle({ width: '34px', height: '20px' });
		await userEvent.click(toggle);
		await expect(toggle).toHaveAttribute('aria-checked', 'true');
		await userEvent.keyboard(' ');
		await expect(toggle).toHaveAttribute('aria-checked', 'false');
	}}
>
	{#snippet template(args)}
		<Toggle {...args} checked={settingsChecked} on:click={() => (settingsChecked = !settingsChecked)} />
	{/snippet}
</Story>

<Story name="Disabled" args={{ disabled: true }} play={checkDisabledToggle}>
	Enable notifications
</Story>

<Story name="Disabled Checked" args={{ disabled: true, checked: true }} play={checkDisabledToggle}>
	Enable notifications
</Story>

<Story
	name="Settings Disabled"
	args={{ appearance: 'settings', disabled: true }}
	play={checkDisabledToggle}
/>

<Story
	name="Settings Disabled Checked"
	args={{ appearance: 'settings', disabled: true, checked: true }}
	play={checkDisabledToggle}
/>

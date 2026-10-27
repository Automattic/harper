<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import SlideDeck from './SlideDeck.svelte';

const { Story } = defineMeta({
	title: 'Components/SlideDeck',
	component: SlideDeck,
	tags: ['autodocs'],
	parameters: {
		layout: 'fullscreen',
	},
	args: {
		title: 'Slide deck',
		lede: 'A simple slide deck example.',
		nextDisabled: false,
		showAside: false,
	},
	argTypes: {
		slideProgress: { control: false },
		onBack: { control: false },
		onNext: { control: false },
		nextDisabled: { control: 'boolean' },
		showAside: { control: 'boolean' },
	},
});
</script>

<script lang="ts">
let step = $state(0);
</script>

<Story name="Default">
	{#snippet template(args)}
		<SlideDeck
			{...args}
			slideProgress={step / 2}
			onBack={() => (step = Math.max(0, step - 1))}
			onNext={() => (step = Math.min(2, step + 1))}
		>
			<p class="mt-4">Slide {step + 1} of 3</p>
		</SlideDeck>
	{/snippet}
</Story>

<Story name="With Aside" args={{ showAside: true }}>
	{#snippet template(args)}
		<SlideDeck
			{...args}
			slideProgress={step / 2}
			onBack={() => (step = Math.max(0, step - 1))}
			onNext={() => (step = Math.min(2, step + 1))}
		>
			<p class="mt-4">Slide {step + 1} of 3</p>
			<div slot="aside" class="grid size-48 place-items-center rounded-lg bg-primary-100 text-primary-900">
				Aside content
			</div>
		</SlideDeck>
	{/snippet}
</Story>

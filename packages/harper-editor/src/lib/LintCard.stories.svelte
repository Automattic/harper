<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import LintCard from './LintCard.svelte';
import { exampleLint, exampleSnippet } from './stories/fixtures.js';

const { Story } = defineMeta({
	title: 'Editor/LintCard',
	component: LintCard,
	tags: ['autodocs'],
	args: {
		lint: exampleLint,
		snippet: exampleSnippet,
		onToggleOpen: () => {},
		focusError: () => {},
		onApply: () => {},
	},
});
</script>

<Story name="Closed" />
<Story name="Open" args={{ open: true, active: true }} />
<Story name="No suggestions" args={{ open: true, lint: { ...exampleLint, suggestions: [] } }} />
<Story name="Dark open" args={{ open: true, active: true }}>
	{#snippet template(args)}
		<div class="dark rounded-xl bg-stone-950 p-4">
			<LintCard {...args} />
		</div>
	{/snippet}
</Story>

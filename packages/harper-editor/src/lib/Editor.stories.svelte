<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import { expect, waitFor } from 'storybook/test';
import EditorExample from './stories/EditorExample.svelte';

const { Story } = defineMeta({
	title: 'Editor/Editor',
	component: EditorExample,
});
</script>

<Story name="With suggestions" play={async ({ canvasElement }) => {
	await waitFor(() => expect(canvasElement.querySelector('.harper-editor-surface')).not.toBeNull(), { timeout: 30000 });
}} />
<Story name="Empty" args={{ content: '' }} />
<Story name="Dark" args={{ dark: true }} />

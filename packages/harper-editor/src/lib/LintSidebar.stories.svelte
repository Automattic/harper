<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import SidebarExample from './stories/SidebarExample.svelte';

const { Story } = defineMeta({
	title: 'Editor/LintSidebar',
	component: SidebarExample,
});
</script>

<Story name="With problems" />
<Story name="All clear" args={{ populated: false }} />
<Story name="Dark with problems" args={{ dark: true }} />
<Story name="Dark all clear" args={{ dark: true, populated: false }} />

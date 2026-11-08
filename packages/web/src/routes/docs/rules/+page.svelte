<script module lang="ts">
export const frontmatter = {
	title: 'Rules',
	description:
		'Browse Harper grammar rules, their descriptions, and whether each rule is enabled by default.',
};
</script>

<PageMetadata metadata={frontmatter} />
<h1 class="mt-0 mb-5 text-[32px]! leading-[1.3]! font-bold! tracking-[-0.02em]! text-(--docs-heading)!">{frontmatter.title}</h1>

<script lang="ts">
import PageMetadata from '$lib/PageMetadata.svelte';
import EditPage from '$lib/docs/EditPage.svelte';
import {
	Table,
	TableBody,
	TableBodyCell,
	TableBodyRow,
	TableHead,
	TableHeadCell,
} from 'components';
import type { PageData } from './$types';

let { data }: { data: PageData } = $props();
</script>

<p>This page is an incomplete list of the various grammatical rules Harper checks for.</p>

<div class="w-full overflow-x-auto" role="region" aria-label="Grammar rules" tabindex="0">
<Table>
	<TableHead class="dark:bg-(--docs-panel) dark:text-(--docs-muted)">
		<TableHeadCell>Name</TableHeadCell>
		<TableHeadCell>Enabled by Default</TableHeadCell>
		<TableHeadCell>Description</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each Object.entries(data.descriptions) as [name, description]}
			<TableBodyRow class="dark:border-(--docs-border) dark:bg-transparent">
				<TableBodyCell class="dark:text-(--docs-heading)">{name}</TableBodyCell>
				<TableBodyCell>{data.defaultConfig[name] ? '✔️' : '❌'}</TableBodyCell>
				<TableBodyCell tdClass="px-6 py-4 font-medium" class="dark:text-(--docs-text)">{@html description.replaceAll('<p>', "").replaceAll('<p />', "")}</TableBodyCell>
			</TableBodyRow>
		{/each}
	</TableBody>
</Table>
</div>
<EditPage sourcePath="docs/rules/+page.svelte" />

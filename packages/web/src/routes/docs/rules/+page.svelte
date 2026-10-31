<script module lang="ts">
export const frontmatter = {
	title: 'Rules',
	description:
		'Browse Harper grammar rules, their descriptions, and whether each rule is enabled by default.',
};
</script>

<script lang="ts">
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

<Table>
	<TableHead>
		<TableHeadCell>Name</TableHeadCell>
		<TableHeadCell>Enabled by Default</TableHeadCell>
		<TableHeadCell>Description</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each Object.entries(data.descriptions) as [name, description]}
			<TableBodyRow>
				<TableBodyCell>{name}</TableBodyCell>
				<TableBodyCell>{data.defaultConfig[name] ? '✔️' : '❌'}</TableBodyCell>
				<TableBodyCell tdClass="px-6 py-4 font-medium">{@html description.replaceAll('<p>', "").replaceAll('<p />', "")}</TableBodyCell>
			</TableBodyRow>
		{/each}
	</TableBody>
</Table>

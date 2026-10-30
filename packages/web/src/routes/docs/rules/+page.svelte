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
import type { LintConfig } from 'harper.js';
import { onMount } from 'svelte';

let descriptions: Record<string, string> = $state({});
let default_config: LintConfig = $state({});

onMount(() => {
	void (async () => {
		const [{ LocalLinter }, { slimBinary }] = await Promise.all([
			import('harper.js'),
			import('harper.js/slimBinary'),
		]);
		const linter = new LocalLinter({ binary: slimBinary });
		try {
			[descriptions, default_config] = await Promise.all([
				linter.getLintDescriptionsHTML(),
				linter.getDefaultLintConfig(),
			]);
		} finally {
			await linter.dispose();
		}
	})();
});
</script>

<p>This page is an incomplete list of the various grammatical rules Harper checks for.</p>

<Table>
	<TableHead>
		<TableHeadCell>Name</TableHeadCell>
		<TableHeadCell>Enabled by Default</TableHeadCell>
		<TableHeadCell>Description</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each Object.entries(descriptions) as [name, description]}
			<TableBodyRow>
				<TableBodyCell>{name}</TableBodyCell>
				<TableBodyCell>{default_config[name] ? '✔️' : '❌'}</TableBodyCell>
				<TableBodyCell tdClass="px-6 py-4 font-medium">{@html description.replaceAll('<p>', "").replaceAll('<p />', "")}</TableBodyCell>
			</TableBodyRow>
		{/each}
	</TableBody>
</Table>

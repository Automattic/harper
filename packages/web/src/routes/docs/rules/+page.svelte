<script module lang="ts">
import {
	Table,
	TableBody,
	TableBodyCell,
	TableBodyRow,
	TableHead,
	TableHeadCell,
} from 'flowbite-svelte';
import { binary, type LintConfig, LocalLinter } from 'harper.js';

export const frontmatter = {
	title: 'Rules',
};

let descriptions: Record<string, string> = $state({});
let default_config: LintConfig = $state({});

let linter = new LocalLinter({ binary });
linter.getLintDescriptionsHTML().then(async (v) => {
	descriptions = v;
});
linter.getDefaultLintConfig().then(async (v) => {
	default_config = v;
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

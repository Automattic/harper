<script lang="ts">
import LintKindChart from '$lib/LintKindChart.svelte';
import {
	Fileupload,
	Table,
	TableBody,
	TableBodyCell,
	TableBodyRow,
	TableHead,
	TableHeadCell,
} from 'flowbite-svelte';
import { binary, type Summary, WorkerLinter } from 'harper.js';

let linter = new WorkerLinter({ binary: binary });
let files = $state<FileList | undefined>();
let summary: Summary | undefined = $state();

$effect(() => {
	(async () => {
		if (files && files.length >= 1) {
			let file = files.item(0);

			let t = await file?.text();

			if (!t) {
				throw new Error('Unable to get text content.');
			}

			await linter.importStatsFile(t);
			summary = await linter.summarizeStats();
		}
	})();
});
</script>

{#if summary}
  <h1>Harper Statistics</h1>

  {#if summary.lint_counts}
    <h2>Most Common Kinds of Corrections</h2>
    <LintKindChart lintCounts={summary.lint_counts}/>
  {/if}

  {#if summary.total_applied}
    <p>In total, {summary.total_applied} corrections were applied.</p>
  {/if}

  {#if summary.misspelled}
    <h2>Most Misspelled Words</h2>
    <Table>
    	<TableHead>
    		<TableHeadCell>Word</TableHeadCell>
    		<TableHeadCell># of Times Misspelled</TableHeadCell>
    	</TableHead>
    	<TableBody>
    		{#each Object.entries(summary.misspelled) as [word, count]}
    			<TableBodyRow>
    				<TableBodyCell>{word}</TableBodyCell>
    				<TableBodyCell>{count}</TableBodyCell>
    			</TableBodyRow>
    		{/each}
    	</TableBody>
    </Table>
    {/if}
{:else}
  <p> Upload your `stats.txt` file to start reflecting on your authorship. </p>
  <Fileupload bind:files={files}/>
{/if}

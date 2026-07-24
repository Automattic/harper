<script lang="ts">
import BarChart from './BarChart.svelte';

// Receive lint counts from the parent component
export let lintCounts: Record<string, number> = {};
</script>

<BarChart data={lintCounts} title="Most Common Kinds of Corrections" label="Number of Corrections Applied" />


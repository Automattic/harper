<script lang="ts">
import AnalyticsPage from '$lib/components/AnalyticsPage.svelte';
import type { PageProps } from './$types';

let { data }: PageProps = $props();
let links = {
	All: '/admin/problematic-lints/all',
	'Last 30 Days': '/admin/problematic-lints/last30days',
	'Last week': '/admin/problematic-lints/lastweek',
	'Last Day': '/admin/problematic-lints/lastday',
};
</script>

<AnalyticsPage data={data.counts} prevPeriodData={data.prevCounts} title="Most Reported Erroring Lint IDs" {links}/>

<script lang="ts">
import AnalyticsPage from '$lib/components/AnalyticsPage.svelte';
import type { PageProps } from './$types';

let { data }: PageProps = $props();
let links = {
	All: '/admin/problematic-lints/all',
	'Last 30 Days': '/admin/problematic-lints/last30days',
	'Last Day': '/admin/problematic-lints/lastday',
};
</script>

<AnalyticsPage data={data.counts} title="Most Reported Erroring Lint IDs" {links}/>

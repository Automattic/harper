<script lang="ts">
import AnalyticsPage from '$lib/components/AnalyticsPage.svelte';
import type { PageProps } from './$types';

let { data }: PageProps = $props();
let links = {
	All: '/admin/ext-site-problems/all',
	'Last 30 Days': '/admin/ext-site-problems/last30days',
	'Last Day': '/admin/ext-site-problems/lastday',
};
</script>

<AnalyticsPage data={data.counts} title="Domains with the most reported problems." {links}/>

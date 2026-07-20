<script lang="ts">
import type { PageProps } from './$types';
import AnalyticsPage from '$lib/components/AnalyticsPage.svelte';

let { data }: PageProps = $props();
let links = {
  All: "/admin/problematic-lints/all",
  ["Last 30 Days"]: "/admin/problematic-lints/last30days",
  ["Last Day"]: "/admin/problematic-lints/lastday",
};

</script>

<AnalyticsPage data={data} title="Most Reported Erroring Lint IDs" {links}/>

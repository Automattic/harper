<script lang="ts">
import type { PageProps } from './$types';
import AnalyticsPage from '$lib/components/AnalyticsPage.svelte';

let { data }: PageProps = $props();
let links = {
  All: "/admin/ext-uninstall-reasons/all",
  ["Last 30 Days"]: "/admin/ext-uninstall-reasons/last30days",
  ["Last Day"]: "/admin/ext-uninstall-reasons/lastday",
};
</script>

<AnalyticsPage data={data} title="Most Common Reasons to Uninstall the Chrome Ext" {links}/>

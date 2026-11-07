<script lang="ts">
import TimeseriesAnalyticsPage from '$lib/components/TimeseriesAnalyticsPage.svelte';
import type { PageProps } from './$types';

let { data }: PageProps = $props();
</script>

<TimeseriesAnalyticsPage heading={data.reason} datapoints={data.points} />

<script lang="ts">
import { Isolate } from 'components';
import LineChart from '$lib/components/LineChart.svelte';
import type { PageProps } from './$types';

let { data }: PageProps = $props();
</script>

<Isolate>
  <h1>{data.lintId}</h1>
  <LineChart title="Reports per day (UTC)" data={data.points} />
</Isolate>

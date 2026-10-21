<script lang="ts">
import { Isolate } from 'components';
import LineChart from '$lib/components/LineChart.svelte';
import type { PageProps } from './$types';

let { data }: PageProps = $props();
</script>

<Isolate>
  <LineChart title="Installs Per Day" data={data.all.map(({date, count}) => { return {date, value: count}})} />
</Isolate>

<script lang="ts">
import { Isolate } from 'components';
import LineChart, { type TimeSeriesPoint } from './LineChart.svelte';

interface Props {
	heading: string;
	datapoints: TimeSeriesPoint[];
}

let { heading, datapoints }: Props = $props();
</script>

<Isolate>
  <h1>{heading}</h1>
  <LineChart title="Reports per day (UTC)" data={datapoints} />
</Isolate>

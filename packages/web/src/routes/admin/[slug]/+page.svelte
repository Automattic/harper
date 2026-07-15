<script lang="ts">
import { GutterCenter, Isolate } from 'components';
import BarChart from '$lib/components/BarChart.svelte';

let { data }: PageProps = $props();
let counts = $derived(data.counts as Record<string, number>);
let entries = $derived(Object.entries(counts).toSorted(([_a, a], [_b, b]) => b - a));
</script>

<Isolate>
  <h1>Most Reported Erroring Lint IDs</h1>

  <div class="flex flex-row [&>a]:px-4">
    <a href="/admin/all">All</a>
    <a href="/admin/last30days">Last 30 Days</a>
    <a href="/admin/lastday">Last Day</a>
  </div>

  <GutterCenter >
    <BarChart data={counts} label="A" title="Most Commonly Complained-About Lint IDs"/>
    
    <table>
    	<thead>
    		<tr>
    			<th>Lint ID</th>
    			<th>Value</th>
    		</tr>
    	</thead>
    	<tbody>
        {#each entries as [key, value] }
    		  <tr>
    		  	<th scope="row">{key}</th>
    		  	<td>{value}</td>
    		  </tr>
        {/each}
    	</tbody>
    </table>
  </GutterCenter>
</Isolate>

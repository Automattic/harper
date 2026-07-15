<script lang="ts">
import { Isolate } from 'components';
import BarChart from '$lib/components/BarChart.svelte';
import type { ProblematicLintRow } from '$lib/db/models/ProblematicLints';
import type { PageProps } from '../$types';

let { data }: PageProps = $props();
let counts = data.counts as Record<string, number>;
let entries = $derived(Object.entries(counts).toSorted(([_a, a], [_b, b]) => b - a));
</script>

<Isolate>
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
    </Isolate>

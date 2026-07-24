<script lang="ts">
    import { GutterCenter, Isolate } from "components";
    import BarChart from "./BarChart.svelte";

type Props = {
  data: Record<string, number>,
  title: string,
  links: Record<string, string>
};

let { data, title, links } = $props();

let counts = $derived(data.counts as Record<string, number>);
let entries = $derived(Object.entries(counts).toSorted(([_a, a], [_b, b]) => b - a));

</script>

<Isolate>
  <h1>{title}</h1>

  <div class="flex flex-row [&>a]:px-4">
    {#each Object.entries(links) as [name, href]}
      <a href={href}>{name}</a>
    {/each}
  </div>

  <GutterCenter >
    <BarChart data={counts} title="Most Commonly Complained-About Lint IDs"/>
    
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

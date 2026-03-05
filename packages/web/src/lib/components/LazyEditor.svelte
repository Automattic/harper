<script lang="ts">
import { Spinner } from 'components';
export let content: string | undefined = undefined;

let editor = import('./Editor.svelte');
let loading = true;

function onReady() {
	loading = false;
}
</script>

{#await editor then { default: Editor}}
  <div class={`flex-row h-full w-full ${loading ? "hidden" : "flex"}`}>
		<Editor content={content} {onReady}/>
  </div>
{/await}

{#if loading}
  <div class="flex h-full max-w-full flex-col items-center justify-center gap-2">
    <Spinner color="green" />
    <p class="text-sm">Loading Harper's grammar engine...</p>
  </div>
{/if}

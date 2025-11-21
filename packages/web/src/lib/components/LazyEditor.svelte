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
  <div class="flex flex-row h-full max-w-full items-center justify-center">
    <Spinner color="green" />
  </div>
{/if}

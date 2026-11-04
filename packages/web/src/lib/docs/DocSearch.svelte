<script lang="ts">
import '@docsearch/css/dist/style.css';
import docsearch from '@docsearch/js';
import { onMount } from 'svelte';
import { docSearchConfig } from './config';

let container: HTMLDivElement;

onMount(() => {
	docsearch({ container, ...docSearchConfig });
});
</script>

<!-- The modal is mounted outside this component. Root variants keep its dark colors scoped to documentation search. -->
<div
	class="min-w-10 flex-1
		[&_.DocSearch-Button]:m-0! [&_.DocSearch-Button]:h-9! [&_.DocSearch-Button]:w-full! [&_.DocSearch-Button]:max-w-[260px]! [&_.DocSearch-Button]:rounded-lg! [&_.DocSearch-Button]:border! [&_.DocSearch-Button]:border-(--docs-border)! [&_.DocSearch-Button]:bg-(--docs-panel)! [&_.DocSearch-Button]:px-3! [&_.DocSearch-Button]:text-(--docs-muted)! [&_.DocSearch-Button]:shadow-none! [&_.DocSearch-Button:hover]:border-(--docs-link)!
		[&_.DocSearch-Button-Placeholder]:text-[13px]! [&_.DocSearch-Search-Icon]:size-4! [&_.DocSearch-Search-Icon]:text-inherit!
		[&_.DocSearch-Button-Key]:border! [&_.DocSearch-Button-Key]:border-(--docs-border)! [&_.DocSearch-Button-Key]:bg-transparent! [&_.DocSearch-Button-Key]:text-(--docs-muted)! [&_.DocSearch-Button-Key]:shadow-none! [&_.DocSearch-Button-Key]:[font-family:inherit]!
		max-[949px]:[&_.DocSearch-Button]:ml-auto! max-[479px]:[&_.DocSearch-Button-Keys]:hidden!
		[html.dark:has(&)]:[--docsearch-text-color:#dfdedd]! [html.dark:has(&)]:[--docsearch-muted-color:#a1a1aa]! [html.dark:has(&)]:[--docsearch-searchbox-focus-background:#18181b]! [html.dark:has(&)]:[--docsearch-modal-background:#202023]! [html.dark:has(&)]:[--docsearch-footer-background:#1c1c1f]! [html.dark:has(&)]:[--docsearch-hit-background:#2a2a2e]! [html.dark:has(&)]:[--docsearch-hit-color:#dfdedd]! [html.dark:has(&)]:[--docsearch-key-background:#2a2a2e]! [html.dark:has(&)]:[--docsearch-key-color:#dfdedd]!"
	bind:this={container}
></div>

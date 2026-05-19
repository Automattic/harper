<script lang="ts">
import '@docsearch/css/dist/style.css';
import docsearch from '@docsearch/js';
import { onMount } from 'svelte';
import { docSearchConfig } from './data';

let container: HTMLDivElement;

onMount(() => {
	if (!container) {
		return;
	}

	docsearch({
		container,
		...docSearchConfig,
	});
});
</script>

<div class="marketing-docsearch" bind:this={container}></div>

<style>
.marketing-docsearch {
	min-width: 11rem;
}

:global(.marketing-docsearch .DocSearch-Button) {
	height: 2.5rem;
	margin: 0;
	border: 0.5px solid var(--marketing-line);
	border-radius: 999px;
	background: rgba(28, 26, 22, 0.04);
	color: var(--marketing-ink-3);
	box-shadow: none;
	font-family: var(--marketing-sans);
	padding: 0 0.75rem;
}

:global(.marketing-docsearch .DocSearch-Button:hover) {
	background: rgba(28, 26, 22, 0.07);
	box-shadow: none;
	color: var(--marketing-ink);
}

:global(.marketing-docsearch .DocSearch-Button-Placeholder) {
	padding: 0 0.65rem 0 0.15rem;
	font-size: 1rem;
	font-weight: 400;
}

:global(.marketing-docsearch .DocSearch-Search-Icon) {
	width: 1.05rem;
	height: 1.05rem;
	color: var(--marketing-ink-3);
	stroke-width: 2;
}

:global(.marketing-docsearch .DocSearch-Button-Keys) {
	display: inline-flex;
	align-items: center;
	min-width: 2.25rem;
	height: 1.35rem;
	justify-content: center;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.3rem;
	background: #fff;
	color: var(--marketing-ink-2);
	font-family: var(--marketing-mono);
	font-size: 0.68rem;
	font-weight: 700;
}

:global(.marketing-docsearch .DocSearch-Button-Keys::before) {
	content: '⌘K';
}

:global(.marketing-docsearch .DocSearch-Button-Key) {
	display: none;
	top: 0;
	width: auto;
	min-width: 1.35rem;
	height: 1.25rem;
	padding: 0 0.32rem;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.25rem;
	background: #fff;
	box-shadow: none;
	color: var(--marketing-ink-2);
	font-family: var(--marketing-mono);
	font-size: 0.62rem;
}

:global(.DocSearch-SearchBar) {
	padding-bottom: calc(var(--docsearch-spacing) / 2);
}

:global(.DocSearch-Form:focus-within) {
	outline: 2px solid #2a6bd8;
	outline-offset: 2px;
	box-shadow: none;
}

:global(.DocSearch-Input:focus),
:global(.DocSearch-Input:focus-visible) {
	border: 0 !important;
	outline: 0 !important;
	box-shadow: none !important;
	-webkit-appearance: none;
	appearance: none;
}

@media (max-width: 760px) {
	.marketing-docsearch {
		min-width: 0;
	}

	:global(.marketing-docsearch .DocSearch-Button) {
		width: 2rem;
		padding: 0;
	}

	:global(.marketing-docsearch .DocSearch-Button-Placeholder),
	:global(.marketing-docsearch .DocSearch-Button-Keys) {
		display: none;
	}
}
</style>

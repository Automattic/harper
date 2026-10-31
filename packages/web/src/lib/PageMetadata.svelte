<script lang="ts">
import type { ResolvedPageMetadata } from './seo';

/** SveltePress owns document titles/descriptions on docs pages; this component owns the other tags. */
interface Props extends ResolvedPageMetadata {
	includeDocumentMetadata?: boolean;
}

let { title, description, canonical, includeDocumentMetadata = true }: Props = $props();
</script>

<svelte:head>
	{#if includeDocumentMetadata}
		<title>{title}</title>
		<meta name="description" content={description} />
	{/if}
	<link rel="canonical" href={canonical} />
	<meta property="og:url" content={canonical} />
	<meta property="og:title" content={title} />
	<meta property="og:description" content={description} />
	<meta name="twitter:title" content={title} />
	<meta name="twitter:description" content={description} />
</svelte:head>

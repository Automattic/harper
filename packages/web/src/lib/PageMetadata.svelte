<script lang="ts">
import { page } from '$app/state';
import { siteConfig } from './docs/config';
import { type MetadataInput, resolvePageMetadata } from './seo';

/** Own document and social metadata; marketing pages omit the site-title suffix. */
interface Props {
	metadata: MetadataInput;
	standaloneTitle?: boolean;
}

let { metadata, standaloneTitle = false }: Props = $props();
const resolved = $derived(
	resolvePageMetadata(page.url.pathname, metadata, siteConfig, standaloneTitle),
);
const title = $derived(resolved.title);
const description = $derived(resolved.description);
const canonical = $derived(resolved.canonical);
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content={description} />
	<link rel="canonical" href={canonical} />
	<meta property="og:url" content={canonical} />
	<meta property="og:title" content={title} />
	<meta property="og:description" content={description} />
	<meta name="twitter:title" content={title} />
	<meta name="twitter:description" content={description} />
</svelte:head>

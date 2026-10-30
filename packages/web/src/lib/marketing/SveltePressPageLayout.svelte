<script>
import siteConfig from 'virtual:sveltepress/site';
import DefaultPageLayout from '@sveltepress/theme-default/PageLayout.svelte';
import { page } from '$app/stores';
import PageMetadata from '$lib/PageMetadata.svelte';
import { resolvePageMetadata } from '$lib/seo';

const { fm, children, heroImage, ...rest } = $props();

const marketingRoutes = new Set(['/', '/get', '/desktop', '/install-browser-extension']);
const isMarketingRoute = $derived(
	marketingRoutes.has($page.url.pathname.replace(/\/$/, '') || '/'),
);
const metadata = $derived(
	resolvePageMetadata($page.url.pathname, fm, siteConfig, isMarketingRoute),
);
</script>

<PageMetadata {...metadata} includeDocumentMetadata={isMarketingRoute} />

{#if isMarketingRoute}
	{@render children?.()}
{:else}
	<DefaultPageLayout {fm} {heroImage} {...rest}>
		{@render children?.()}
	</DefaultPageLayout>
{/if}

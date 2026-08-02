<script>
import DefaultPageLayout from '@sveltepress/theme-default/PageLayout.svelte';
import { page } from '$app/stores';

const { fm, children, heroImage, ...rest } = $props();

const marketingRoutes = new Set(['/', '/get', '/desktop']);
const isMarketingRoute = $derived(
	marketingRoutes.has($page.url.pathname.replace(/\/$/, '') || '/'),
);
</script>

{#if isMarketingRoute}
	{@render children?.()}
{:else}
	<DefaultPageLayout {fm} {heroImage} {...rest}>
		{@render children?.()}
	</DefaultPageLayout>
{/if}

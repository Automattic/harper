<script>
import DefaultGlobalLayout from '@sveltepress/theme-default/GlobalLayout.svelte';
import { page } from '$app/stores';

const { children, ...rest } = $props();

const marketingRoutes = new Set(['/', '/get', '/desktop']);
const isMarketingRoute = $derived(
	marketingRoutes.has($page.url.pathname.replace(/\/$/, '') || '/'),
);
</script>

{#if isMarketingRoute}
	{@render children?.()}
{:else}
	<DefaultGlobalLayout {...rest}>
		{@render children?.()}
	</DefaultGlobalLayout>
{/if}

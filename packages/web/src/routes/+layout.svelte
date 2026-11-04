<script lang="ts">
import '../app.css';

import { AutomatticLogo, GutterCenter, Link } from 'components';
import posthog from 'posthog-js';
import { onMount, type Snippet } from 'svelte';
import { browser } from '$app/environment';
import { page } from '$app/state';
import PageLayout from '$lib/docs/PageLayout.svelte';
import SiteLayout from '$lib/docs/SiteLayout.svelte';

let { children }: { children: Snippet } = $props();

onMount(() => {
	if (browser) {
		posthog.init('phc_ghFPi5nkwgxTGU9VEHflX8QCXlfrdxFD4skfb6lpH4y', {
			api_host: 'https://us.i.posthog.com',
			persistence: 'sessionStorage',
			person_profiles: 'always',
		});
	}
});

let names = ['Grammar Guru', 'Grammar Checker', 'Grammar Savior'];
let displayName = names[Math.floor(Math.random() * names.length)];

const isMarketingRoute = $derived(
	['/', '/get', '/desktop', '/install-browser-extension'].includes(
		page.url.pathname.replace(/\/$/, '') || '/',
	),
);
</script>

{#if isMarketingRoute}
	{@render children()}
{:else}
	<SiteLayout>
	<div class="flex flex-col h-full">
		<div class="flex-1">
			<GutterCenter>
				<PageLayout>{@render children()}</PageLayout>
			</GutterCenter>
		</div>

		<div class="w-full flex flex-row justify-center h-12">
			<Link href="https://automattic.com/">
				<div class="flex items-center">
					An
					<div class="inline-block"><AutomatticLogo height="32px" width="140px" /></div>
					{displayName}
				</div>
			</Link>
		</div>
	</div>
	</SiteLayout>
{/if}

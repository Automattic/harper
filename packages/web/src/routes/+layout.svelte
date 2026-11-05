<script lang="ts">
import '../app.css';

import { AutomatticLogo, GutterCenter, Link } from 'components';
import posthog from 'posthog-js';
import { onMount, type Snippet, setContext } from 'svelte';
import { browser } from '$app/environment';
import { page } from '$app/state';
import DocLayout from '$lib/docs/DocLayout.svelte';
import PageLayout from '$lib/docs/PageLayout.svelte';
import ThemeController, { THEME_CONTEXT } from '$lib/docs/ThemeController';

let { children }: { children: Snippet } = $props();

const theme = setContext(THEME_CONTEXT, new ThemeController());
onMount(() => theme.start());

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
	<DocLayout>
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
	</DocLayout>
{/if}

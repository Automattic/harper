<script lang="ts">
import '../app.css';

import { AutomatticLogo, GutterCenter, Link } from 'components';
import posthog from 'posthog-js';
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import { page } from '$app/stores';

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

$: isMarketingRoute = ['/', '/get', '/desktop'].includes(
	$page.url.pathname.replace(/\/$/, '') || '/',
);
</script>

{#if isMarketingRoute}
	<slot />
{:else}
	<div class="flex flex-col h-full">
		<div class="flex-1">
			<GutterCenter>
				<slot />
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
{/if}

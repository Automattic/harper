<script lang="ts">
import '../app.css';

import { Link } from 'components';
import posthog from 'posthog-js';
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import AutomatticLogo from '$lib/components/AutomatticLogo.svelte';
import GutterCenter from '$lib/components/GutterCenter.svelte';

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
</script>

<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link
	href="https://fonts.googleapis.com/css2?family=Domine:wght@400..700&family=IBM+Plex+Serif:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&display=swap"
	rel="stylesheet"
/>

<link
	href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap"
	rel="stylesheet"
/>

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

<script lang="ts">
import '../app.css';
import AutomatticLogo from '$lib/AutomatticLogo.svelte';
import GutterCenter from '$lib/GutterCenter.svelte';

let names = ['Grammar Guru', 'Grammar Checker', 'Grammar Savior'];
let displayName = names[Math.floor(Math.random() * names.length)];
</script>

<link
	href="https://fonts.googleapis.com/css2?family=Atkinson+Hyperlegible:ital,wght@0,400;0,700;1,400;1,700&display=swap"
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

	<div class="w-full flex flex-row justify-center">
		<a href="https://automattic.com/">
			<div class="flex items-center">
				An
				<div class="inline-block"><AutomatticLogo height="32px" width="140px" /></div>
				{displayName}
			</div>
		</a>
	</div>
</div>

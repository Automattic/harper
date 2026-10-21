<script lang="ts">
import { onDestroy, onMount } from 'svelte';
import { Client } from '$lib/client';

export let bundleId = '';
export let name: string | undefined = undefined;
export let className = '';

let element: HTMLDivElement;
let iconDataUrl = '';
let iconRequestId = 0;
let isVisible = false;
let observer: IntersectionObserver | undefined;

$: fallbackLabel = fallbackInitial(name ?? bundleId);
$: if (isVisible) {
	void loadIcon(bundleId);
}

onMount(() => {
	if (!('IntersectionObserver' in window)) {
		isVisible = true;
		return;
	}

	observer = new IntersectionObserver(
		(entries) => {
			if (entries.some((entry) => entry.isIntersecting)) {
				isVisible = true;
				observer?.disconnect();
				observer = undefined;
			}
		},
		{ rootMargin: '80px' },
	);

	observer.observe(element);
});

onDestroy(() => {
	observer?.disconnect();
});

async function loadIcon(nextBundleId: string) {
	const requestId = ++iconRequestId;
	const trimmedBundleId = nextBundleId.trim();
	iconDataUrl = '';

	if (!trimmedBundleId) {
		return;
	}

	try {
		const nextIconDataUrl = await Client.getApplicationIconDataUrl(trimmedBundleId);

		if (requestId === iconRequestId && bundleId.trim() === trimmedBundleId) {
			iconDataUrl = nextIconDataUrl;
		}
	} catch {
		// Keep the fallback when an app icon cannot be loaded.
	}
}

function fallbackInitial(value: string) {
	return Array.from(value.trim())[0]?.toUpperCase() ?? '?';
}
</script>

<div bind:this={element} class={`app-icon ${className}`} style="--app-tint: #6b6f78" aria-hidden="true">
  {#if iconDataUrl}
    <img src={iconDataUrl} alt="" />
  {:else}
    {fallbackLabel}
  {/if}
</div>

<script lang="ts">
import { Client } from '$lib/client';

export let bundleId = '';
export let name: string | undefined = undefined;
export let className = '';

let iconDataUrl = '';
let iconRequestId = 0;

$: fallbackLabel = fallbackInitial(name ?? bundleId);
$: void loadIcon(bundleId);

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

<div class={`app-icon ${className}`} style="--app-tint: #6b6f78" aria-hidden="true">
  {#if iconDataUrl}
    <img src={iconDataUrl} alt="" />
  {:else}
    {fallbackLabel}
  {/if}
</div>

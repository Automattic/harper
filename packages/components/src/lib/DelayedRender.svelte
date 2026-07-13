<script lang="ts">
import type { Snippet } from 'svelte';

/**
 * Renders arbitrary child content after `active` remains true for `delayMs` milliseconds.
 * Pending renders are cancelled and visible content is hidden immediately when `active` becomes false.
 */
let {
	active = false,
	delayMs = 0,
	children,
}: {
	active?: boolean;
	delayMs?: number;
	children?: Snippet;
} = $props();

let visible = $state(false);
let timeout: ReturnType<typeof setTimeout> | null = null;

/** Cancel the pending delayed render without changing currently rendered content. */
function clearPendingDelay() {
	if (timeout == null) {
		return;
	}

	clearTimeout(timeout);
	timeout = null;
}

$effect(() => {
	clearPendingDelay();

	if (!active) {
		visible = false;
		return;
	}

	timeout = setTimeout(() => {
		visible = true;
		timeout = null;
	}, delayMs);

	return clearPendingDelay;
});
</script>

{#if visible}
	{@render children?.()}
{/if}

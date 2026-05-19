<script lang="ts">
export let href = '';
export let kind: 'primary' | 'secondary' = 'primary';
export let size: 'md' | 'lg' = 'md';
export let disabled = false;
export let label = '';
</script>

{#if disabled}
	<span
		class="pill {kind} {size} disabled"
		role="link"
		aria-disabled="true"
		aria-label={label || undefined}
	>
		<slot name="icon" />
		<slot />
	</span>
{:else}
	<a class="pill {kind} {size}" {href} aria-label={label || undefined}>
		<slot name="icon" />
		<slot />
	</a>
{/if}

<style>
.pill {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 0.5rem;
	height: 2.25rem;
	padding: 0 1rem;
	border-radius: 999px;
	font-family: var(--marketing-sans);
	font-size: 0.82rem;
	font-weight: 700;
	text-decoration: none;
	white-space: nowrap;
	transition:
		transform 120ms ease,
		background-color 160ms ease,
		border-color 160ms ease,
		color 160ms ease;
}

.pill:hover {
	transform: translateY(-1px);
	text-decoration: none;
}

.lg {
	height: 2.875rem;
	padding: 0 1.4rem;
	font-size: 0.94rem;
}

.primary {
	border: 0.5px solid rgba(0, 0, 0, 0.4);
	background: var(--marketing-ink);
	color: #fffdfa;
}

.secondary {
	border: 0.5px solid var(--marketing-line);
	background: rgba(28, 26, 22, 0.04);
	color: var(--marketing-ink);
}

.disabled {
	cursor: default;
	opacity: 0.7;
}

.disabled:hover {
	transform: none;
}
</style>

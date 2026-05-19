<script lang="ts">
import HarperMark from './HarperMark.svelte';

export let word: string;
export let fix: string;
</script>

<div class="suggestion">
	<HarperMark size={11} />
	<del>{word}</del>
	<span>-></span>
	<strong>{fix}</strong>
</div>

<style>
.suggestion {
	position: absolute;
	right: 0.75rem;
	bottom: 0.65rem;
	display: inline-flex;
	align-items: center;
	gap: 0.4rem;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.5rem;
	background: #fbfaf6;
	box-shadow: 0 6px 16px rgba(28, 26, 22, 0.16);
	padding: 0.32rem 0.55rem;
	color: var(--marketing-ink);
	font-size: 0.68rem;
}

del,
strong {
	font-family: var(--marketing-mono);
}

del,
span {
	color: var(--marketing-ink-3);
}
</style>

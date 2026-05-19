<script lang="ts">
export let color = 'var(--marketing-amber)';
</script>

<div class="section-kicker" style:color>
	<slot />
</div>

<style>
.section-kicker {
	font-family: var(--marketing-mono);
	font-size: 0.69rem;
	font-weight: 700;
	letter-spacing: 0.16em;
	text-transform: uppercase;
}
</style>

<script lang="ts">
export let color = '#c96442';
export let thickness = 2;
</script>

<span
	class="squiggle"
	style:--squiggle-color={color}
	style:--squiggle-thickness={`${thickness}px`}
>
	<slot />
</span>

<style>
.squiggle {
	--squiggle-color: #c96442;
	--squiggle-thickness: 2px;
	position: relative;
	display: inline-block;
	color: inherit;
	padding-bottom: 0.16em;
	text-decoration: none;
	vertical-align: baseline;
}

.squiggle::after {
	position: absolute;
	right: 0;
	bottom: 0;
	left: 0;
	height: max(4px, calc(var(--squiggle-thickness) * 2));
	background: var(--squiggle-color);
	content: '';
	pointer-events: none;
	-webkit-mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 4' preserveAspectRatio='none'%3E%3Cpath d='M0 3 Q1 0 2 3 T4 3 T6 3 T8 3' fill='none' stroke='black' stroke-width='2' stroke-linecap='round'/%3E%3C/svg%3E");
	mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 4' preserveAspectRatio='none'%3E%3Cpath d='M0 3 Q1 0 2 3 T4 3 T6 3 T8 3' fill='none' stroke='black' stroke-width='2' stroke-linecap='round'/%3E%3C/svg%3E");
	-webkit-mask-position: left bottom;
	mask-position: left bottom;
	-webkit-mask-repeat: repeat-x;
	mask-repeat: repeat-x;
	-webkit-mask-size: 8px 100%;
	mask-size: 8px 100%;
}
</style>

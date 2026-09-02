<script lang="ts">
import { createEventDispatcher } from 'svelte';
import type { HTMLButtonAttributes } from 'svelte/elements';

export let type: HTMLButtonAttributes['type'] = 'button';
export let disabled: HTMLButtonAttributes['disabled'] = undefined;
export let danger = false;
export let className = '';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};
const dispatch = createEventDispatcher<{ click: MouseEvent }>();

$: ({ class: restClass, ...restProps } = $$restProps);
const baseClasses =
	'size-[26px] flex-none grid place-items-center rounded-[6px] border-0 bg-transparent text-[var(--settings-ink-3,#807a6e)] [&>svg]:size-[13px] [&>svg]:flex-none';
const hoverClasses = 'hover:bg-black/[0.06] hover:text-[var(--settings-ink,#1c1a16)]';
const dangerHoverClasses = 'hover:bg-[#b42318]/[0.08] hover:text-[#b42318]';

$: classes = [
	'icon-button',
	baseClasses,
	danger && 'danger',
	danger ? dangerHoverClasses : hoverClasses,
	restClass,
	className,
]
	.filter(Boolean)
	.join(' ');
</script>

<button
	class={classes}
	{type}
	{disabled}
	{...restProps}
	on:click={(event) => dispatch('click', event)}
>
	<slot />
</button>

<script lang="ts">
import { Toggle as FlowbiteToggle } from 'flowbite-svelte';
import { createEventDispatcher } from 'svelte';
import type { HTMLButtonAttributes } from 'svelte/elements';

export let checked = false;
export let disabled: HTMLButtonAttributes['disabled'] = undefined;
export let appearance: 'default' | 'settings' = 'default';
export let className = '';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};
const dispatch = createEventDispatcher<{ click: MouseEvent }>();

$: ({ class: restClass, ...restProps } = $$restProps);
const baseClasses =
	'h-5 w-[34px] flex-none rounded-full border-0 p-0.5 shadow-[inset_0_0.5px_1px_rgb(0_0_0_/_10%)]';
const uncheckedClasses = 'bg-[#d4cfc4]';
const checkedClasses = 'bg-[var(--settings-accent,#2a6bd8)]';
const thumbClasses =
	'block size-4 rounded-full bg-[#fff] shadow-[0_1px_2px_rgb(0_0_0_/_20%),0_0.5px_0_rgb(0_0_0_/_10%)] transition-transform duration-150 ease-[ease]';

$: classes = [
	'toggle',
	baseClasses,
	checked && 'checked',
	checked ? checkedClasses : uncheckedClasses,
	restClass,
	className,
]
	.filter(Boolean)
	.join(' ');
</script>

{#if appearance === 'settings'}
	<button
		class={classes}
		type="button"
		role="switch"
		aria-checked={checked}
		{disabled}
		{...restProps}
		on:click={(event) => dispatch('click', event)}
	>
		<span class={`${thumbClasses} ${checked ? 'translate-x-3.5' : 'translate-x-0'}`}></span>
	</button>
{:else}
	<FlowbiteToggle bind:checked {disabled} class={restClass} {...restProps}>
		<slot />
	</FlowbiteToggle>
{/if}

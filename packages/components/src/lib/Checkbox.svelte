<script lang="ts">
import { Checkbox as FlowbiteCheckbox } from 'flowbite-svelte';
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
	'size-4 flex-none grid place-items-center rounded border p-0 text-white [&>svg]:size-[11px] [&>svg]:flex-none';
const uncheckedClasses = 'border-black/25 bg-[#fff]';
const checkedClasses = 'border-black/15 bg-[var(--settings-accent,#2a6bd8)]';

$: classes = [
	'checkbox',
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
		role="checkbox"
		aria-checked={checked}
		{disabled}
		{...restProps}
		on:click={(event) => dispatch('click', event)}
	>
		<slot />
	</button>
{:else}
	<FlowbiteCheckbox bind:checked {disabled} class={restClass} {...restProps}>
		<slot />
	</FlowbiteCheckbox>
{/if}

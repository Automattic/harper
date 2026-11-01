<script lang="ts">
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
	'h-5 w-[34px] flex-none rounded-full border-0 p-0.5 shadow-[inset_0_0.5px_1px_rgb(0_0_0_/_10%)] disabled:cursor-not-allowed disabled:opacity-50 disabled:grayscale';
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
	<label
		class="flex items-center text-sm font-medium {disabled ? 'cursor-not-allowed text-gray-400 dark:text-gray-500' : 'text-gray-900 dark:text-gray-300'} {restClass ?? ''} {className}"
	>
		<input class="peer sr-only" type="checkbox" role="switch" bind:checked {disabled} {...restProps} />
		<span
			aria-hidden="true"
			class="relative me-3 h-6 w-11 shrink-0 rounded-full bg-gray-200 after:absolute after:start-[2px] after:top-0.5 after:h-5 after:w-5 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:bg-primary-600 peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:ring-4 peer-focus:ring-primary-300 peer-disabled:cursor-not-allowed peer-disabled:opacity-50 peer-disabled:grayscale dark:bg-gray-700 dark:peer-focus:ring-primary-800 rtl:peer-checked:after:-translate-x-full motion-reduce:after:transition-none"
		></span>
		<slot />
	</label>
{/if}

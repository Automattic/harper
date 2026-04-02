<script lang="ts">
import type { SelectHTMLAttributes } from 'svelte/elements';

type SelectSize = 'sm' | 'md' | 'lg';
type SelectItem = {
	value: SelectHTMLAttributes['value'];
	name?: string;
	label?: string;
	disabled?: boolean;
	selected?: boolean;
};

export let size: SelectSize = 'md';
export let items: SelectItem[] | undefined = undefined;
export let className: string | undefined = undefined;
export let value: SelectHTMLAttributes['value'] = undefined;

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

const baseClasses =
	'rounded-lg border border-cream-200 bg-white shadow-sm outline-none transition focus:ring-2 focus:ring-primary-300 focus:border-cream-300 dark:border-cream-700 dark:bg-cream-900 dark:text-white dark:focus:border-cream-600 dark:focus:ring-primary-600 text-left';
const sizeClasses: Record<SelectSize, string> = {
	sm: 'pl-3 pr-8 py-2 text-sm',
	md: 'pl-3 pr-8 py-2.5 text-sm',
	lg: 'pl-4 pr-8 py-3 text-base',
};

$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = [baseClasses, sizeClasses[size] ?? sizeClasses.md, restClass, className]
	.filter(Boolean)
	.join(' ');
</script>

<select class={classes} bind:value {...restProps}>
	{#if items?.length}
		{#each items as item (item.value)}
			<option value={item.value} disabled={item.disabled} selected={item.selected}>
				{item.name ?? item.label ?? item.value}
			</option>
		{/each}
	{:else}
		<slot />
	{/if}
</select>

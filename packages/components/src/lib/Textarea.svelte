<script lang="ts">
import type { HTMLTextareaAttributes } from 'svelte/elements';

export let className: string | undefined = undefined;
export let value: HTMLTextareaAttributes['value'] = undefined;
export let rows: HTMLTextareaAttributes['rows'] = undefined;
export let cols: HTMLTextareaAttributes['cols'] = undefined;

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

const baseClasses =
	'rounded-lg border border-cream-200 bg-white text-gray-900 shadow-sm placeholder-gray-500 outline-none transition focus:ring-2 focus:ring-primary-300 focus:border-cream-300 dark:border-cream-700 dark:bg-cream-900 dark:text-white dark:placeholder-cream-200 dark:focus:border-cream-600 dark:focus:ring-primary-600';

// Align with Svelte's `class` handling while allowing `className` as an alias.
$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = [baseClasses, restClass, className].filter(Boolean).join(' ');
</script>

<textarea class={classes} bind:value rows={rows} cols={cols} {...restProps}>
	<slot />
</textarea>

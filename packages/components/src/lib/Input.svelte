<script lang="ts">
import type { InputHTMLAttributes } from 'svelte/elements';

type InputSize = 'sm' | 'md' | 'lg';

export let type: InputHTMLAttributes['type'] = 'text';
export let value: InputHTMLAttributes['value'] = undefined;
export let placeholder: InputHTMLAttributes['placeholder'] = undefined;
export let className: string | undefined = undefined;
export let size: InputSize = 'md';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

const baseClasses =
	'rounded-lg border border-cream-200 bg-white text-gray-900 placeholder-gray-500 shadow-sm outline-none transition focus:ring-2 focus:ring-primary-300 focus:border-cream-300 dark:border-cream-700 dark:bg-cream-900 dark:text-white dark:placeholder-cream-200 dark:focus:border-cream-600 dark:focus:ring-primary-600';
const sizeClasses: Record<InputSize, string> = {
	sm: 'px-3 py-2 text-sm',
	md: 'px-3 py-2.5 text-sm',
	lg: 'px-4 py-3 text-base',
};

$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = [baseClasses, sizeClasses[size] ?? sizeClasses.md, restClass, className]
	.filter(Boolean)
	.join(' ');
</script>

<input
	class={classes}
	type={type}
	placeholder={placeholder}
	bind:value
	{...restProps}
/>

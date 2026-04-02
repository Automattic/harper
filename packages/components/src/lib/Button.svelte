<script lang="ts">
import { createEventDispatcher } from 'svelte';
import type { AnchorHTMLAttributes, ButtonHTMLAttributes } from 'svelte/elements';
import Link from './Link.svelte';

type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';
type ButtonColor = 'primary' | 'light' | 'gray' | 'white' | 'dark';

export let size: ButtonSize = 'md';
export let color: ButtonColor | string = 'primary';
export let textColor: string | undefined = undefined;
export let pill = false;
export let href: AnchorHTMLAttributes['href'] = undefined;
export let target: AnchorHTMLAttributes['target'] = undefined;
export let rel: AnchorHTMLAttributes['rel'] = undefined;
export let type: ButtonHTMLAttributes['type'] = 'button';
export let disabled: boolean | undefined = undefined;
// Alias for the `class` attribute since `class` is a reserved TS keyword
export let className: string | undefined = undefined;

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};
const dispatch = createEventDispatcher();

const sizeClasses: Record<ButtonSize, string> = {
	xs: 'px-3 py-2 text-xs',
	sm: 'px-3 py-2 text-sm',
	md: 'px-4 py-2.5 text-sm',
	lg: 'px-5 py-3 text-base',
};

const colorClasses: Record<ButtonColor, string> = {
	primary:
		'text-white bg-primary-600 hover:bg-primary-700 focus:ring-primary-300 dark:bg-primary-500 dark:hover:bg-primary-600 dark:focus:ring-primary-700',
	light:
		'text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700',
	gray: 'text-white bg-gray-800 hover:bg-gray-900 focus:ring-gray-300 dark:bg-gray-700 dark:hover:bg-gray-800 dark:focus:ring-gray-900',
	white:
		'text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700',
	dark: 'text-white bg-gray-900 hover:bg-black focus:ring-gray-300 dark:bg-gray-800 dark:hover:bg-black dark:focus:ring-gray-900',
};
const baseClasses =
	'cursor-pointer inline-flex items-center gap-2 justify-center font-medium text-center transition-colors focus:outline-none focus:ring-4 disabled:opacity-50 disabled:cursor-not-allowed';

$: toneClass = colorClasses[color as ButtonColor] ?? colorClasses.primary;
$: shapeClass = pill ? 'rounded-full' : 'rounded-lg';
$: sizeClass = sizeClasses[size] ?? sizeClasses.md;
$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = [baseClasses, shapeClass, sizeClass, toneClass, restClass, className]
	.filter(Boolean)
	.join(' ');

$: colorOverride = colorClasses[color as ButtonColor] == null ? color : undefined;
$: inlineStyle =
	colorOverride || textColor
		? [
				colorOverride ? `background-color: ${colorOverride} !important;` : null,
				textColor ? `color: ${textColor} !important;` : null,
			]
				.filter(Boolean)
				.join(' ')
		: undefined;

function handleClick(event: MouseEvent) {
	if (disabled) {
		event.preventDefault();
		event.stopPropagation();
		return;
	}

	dispatch('click', event);
}
</script>

{#if href}
	<Link
		class={classes}
		style={inlineStyle}
		href={disabled ? undefined : href}
		aria-disabled={disabled}
		role={disabled ? 'link' : undefined}
		tabindex={disabled ? -1 : undefined}
		rel={rel}
		target={target}
		on:click={handleClick}
		{...restProps}
	>
		<slot />
	</Link>
{:else}
	<button
		class={classes}
		type={type}
		{disabled}
		{...restProps}
		style={inlineStyle}
		on:click={handleClick}
	>
		<slot />
	</button>
{/if}

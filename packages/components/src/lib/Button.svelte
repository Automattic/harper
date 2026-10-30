<script lang="ts">
import { createEventDispatcher, type Snippet } from 'svelte';
import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';
import CheckIcon from './icons/CheckIcon.svelte';

type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';
type ButtonColor = 'primary' | 'gray' | 'white' | 'dark';
/** Mutually exclusive feedback states, controlled by the caller. */
type ButtonState = 'idle' | 'loading' | 'success';

/** Button or link attributes, snippet content, and guarded native activation callbacks. */
interface ButtonProps
	extends Omit<HTMLButtonAttributes & HTMLAnchorAttributes, 'color' | 'onclick' | 'ondblclick'> {
	size?: ButtonSize;
	color?: ButtonColor | string;
	textColor?: string;
	pill?: boolean;
	/** Loading blocks activation; success remains interactive. The caller controls transitions and resets. */
	state?: ButtonState;
	unstyled?: boolean;
	/** Alias for the native `class` attribute; both values are included. */
	className?: string;
	children?: Snippet;
	onclick?: (event: MouseEvent) => void;
	ondblclick?: (event: MouseEvent) => void;
}

let {
	size = 'md',
	color = 'primary',
	textColor,
	pill = false,
	href,
	target,
	rel,
	type = 'button',
	disabled,
	state = 'idle',
	unstyled = false,
	className,
	class: restClass,
	children,
	onclick,
	ondblclick,
	...restProps
}: ButtonProps = $props();

// Retain legacy component events while callers migrate to callback props.
const dispatch = createEventDispatcher<{ click: Event; dblclick: Event }>();

const sizeClasses: Record<ButtonSize, string> = {
	xs: 'px-3 py-2 text-xs',
	sm: 'px-3 py-2 text-sm',
	md: 'px-4 py-2.5 text-sm',
	lg: 'px-5 py-3 text-base',
};

const colorClasses: Record<ButtonColor, string> = {
	primary:
		'text-white bg-primary-600 hover:bg-primary-700 focus:ring-primary-300 dark:bg-primary-500 dark:hover:bg-primary-600 dark:focus:ring-primary-700',
	gray: 'text-white bg-gray-800 hover:bg-gray-900 focus:ring-gray-300 dark:bg-gray-700 dark:hover:bg-gray-800 dark:focus:ring-gray-900',
	white:
		'text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700',
	dark: 'text-white bg-gray-900 hover:bg-black focus:ring-gray-300 dark:bg-gray-800 dark:hover:bg-black dark:focus:ring-gray-900',
};
const baseClasses =
	'cursor-pointer inline-flex items-center gap-2 justify-center font-medium text-center transition-colors focus:outline-none focus:ring-4 disabled:opacity-50 disabled:cursor-not-allowed';

const effectiveDisabled = $derived(disabled || state === 'loading');
const resolvedRel = $derived(target === '_blank' && !rel ? 'noreferrer noopener' : rel);
const toneClass = $derived(colorClasses[color as ButtonColor]);
const classes = $derived(
	[
		// Retain the Link component's styling for existing href callers.
		href && 'hover:underline text-primary dark:text-white',
		!unstyled && baseClasses,
		!unstyled && (pill ? 'rounded-full' : 'rounded-lg'),
		!unstyled && (sizeClasses[size] ?? sizeClasses.md),
		!unstyled && (toneClass ?? colorClasses.primary),
		!unstyled && href && effectiveDisabled && 'opacity-50 cursor-not-allowed',
		restClass,
		className,
	]
		.filter(Boolean)
		.join(' '),
);

const colorOverride = $derived(!unstyled && toneClass == null ? color : undefined);
const inlineStyle = $derived(
	colorOverride || textColor
		? [
				colorOverride ? `background-color: ${colorOverride} !important;` : null,
				textColor ? `color: ${textColor} !important;` : null,
			]
				.filter(Boolean)
				.join(' ')
		: undefined,
);

/** Block disabled/loading activation before invoking callbacks and legacy component events. */
function handleActivation(event: MouseEvent) {
	if (effectiveDisabled) {
		event.preventDefault();
		event.stopPropagation();
		return;
	}

	if (event.type === 'dblclick') {
		ondblclick?.(event);
		dispatch('dblclick', event);
	} else {
		onclick?.(event);
		dispatch('click', event);
	}
}
</script>

<svelte:element
	this={href ? 'a' : 'button'}
	{...restProps}
	class={classes}
	style={href && restProps.style !== undefined ? restProps.style : inlineStyle}
	type={href ? undefined : type}
	disabled={href ? undefined : effectiveDisabled}
	href={effectiveDisabled ? undefined : href}
	target={href ? target : undefined}
	rel={href ? resolvedRel : undefined}
	aria-busy={state === 'loading' ? true : restProps['aria-busy']}
	aria-disabled={href && effectiveDisabled ? true : restProps['aria-disabled']}
	role={href && effectiveDisabled ? 'link' : restProps.role}
	tabindex={href && effectiveDisabled ? -1 : restProps.tabindex}
	onclick={handleActivation}
	ondblclick={handleActivation}
>
	{#if state === 'loading'}
		<svg class="size-4 shrink-0 animate-spin motion-reduce:animate-none" viewBox="0 0 20 20" fill="none" aria-hidden="true">
			<circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" opacity="0.25" />
			<path d="M10 2a8 8 0 0 1 8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
		</svg>
	{:else if state === 'success'}
		<CheckIcon className="size-4 shrink-0" />
	{/if}
	{@render children?.()}
</svelte:element>
<!-- Outside the busy control so loading announcements are not deferred. -->
<span class="sr-only" role="status">{state === 'loading' ? 'Loading' : state === 'success' ? 'Success' : ''}</span>

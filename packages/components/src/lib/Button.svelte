<script lang="ts">
import { createEventDispatcher } from 'svelte';
import type { HTMLAnchorAttributes, HTMLAttributes, HTMLButtonAttributes } from 'svelte/elements';
import CheckIcon from './icons/CheckIcon.svelte';

type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';
type ButtonColor = 'primary' | 'light' | 'gray' | 'white' | 'dark';
/** Mutually exclusive feedback states, controlled by the caller. */
type ButtonState = 'idle' | 'loading' | 'success';

export let size: ButtonSize = 'md';
export let color: ButtonColor | string = 'primary';
export let textColor: string | undefined = undefined;
export let pill = false;
export let href: HTMLAnchorAttributes['href'] = undefined;
export let target: HTMLAnchorAttributes['target'] = undefined;
export let rel: HTMLAnchorAttributes['rel'] = undefined;
export let type: HTMLButtonAttributes['type'] = 'button';
export let disabled: boolean | undefined = undefined;
/** Loading blocks activation; success remains interactive. The caller controls transitions and resets. */
export let state: ButtonState = 'idle';
export let unstyled = false;
// Alias for the `class` attribute since `class` is a reserved TS keyword
export let className: string | undefined = undefined;

let restClass: string | undefined;
let restProps: HTMLAttributes<HTMLElement> = {};
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
	light:
		'text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700',
	gray: 'text-white bg-gray-800 hover:bg-gray-900 focus:ring-gray-300 dark:bg-gray-700 dark:hover:bg-gray-800 dark:focus:ring-gray-900',
	white:
		'text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700',
	dark: 'text-white bg-gray-900 hover:bg-black focus:ring-gray-300 dark:bg-gray-800 dark:hover:bg-black dark:focus:ring-gray-900',
};
const baseClasses =
	'cursor-pointer inline-flex items-center gap-2 justify-center font-medium text-center transition-colors focus:outline-none focus:ring-4 disabled:opacity-50 disabled:cursor-not-allowed';

$: effectiveDisabled = disabled || state === 'loading';
$: resolvedRel = target === '_blank' && !rel ? 'noreferrer noopener' : rel;
$: toneClass = colorClasses[color as ButtonColor];
$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = [
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
	.join(' ');

$: colorOverride = !unstyled && toneClass == null ? color : undefined;
$: inlineStyle =
	colorOverride || textColor
		? [
				colorOverride ? `background-color: ${colorOverride} !important;` : null,
				textColor ? `color: ${textColor} !important;` : null,
			]
				.filter(Boolean)
				.join(' ')
		: undefined;

/** Guard native button/link activation before forwarding the existing Svelte component events. */
function handleActivation(event: Event) {
	if (effectiveDisabled) {
		event.preventDefault();
		event.stopPropagation();
		return;
	}

	dispatch(event.type === 'dblclick' ? 'dblclick' : 'click', event);
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
	on:click={handleActivation}
	on:dblclick={handleActivation}
>
	{#if state === 'loading'}
		<svg class="size-4 shrink-0 animate-spin motion-reduce:animate-none" viewBox="0 0 20 20" fill="none" aria-hidden="true">
			<circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" opacity="0.25" />
			<path d="M10 2a8 8 0 0 1 8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
		</svg>
	{:else if state === 'success'}
		<CheckIcon className="size-4 shrink-0" />
	{/if}
	<slot />
</svelte:element>
<!-- Outside the busy control so loading announcements are not deferred. -->
<span class="sr-only" role="status">{state === 'loading' ? 'Loading' : state === 'success' ? 'Success' : ''}</span>

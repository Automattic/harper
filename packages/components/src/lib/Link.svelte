<script lang="ts">
import { createEventDispatcher } from 'svelte';
import type { AnchorHTMLAttributes } from 'svelte/elements';

export let href: AnchorHTMLAttributes['href'] = undefined;
export let target: AnchorHTMLAttributes['target'] = undefined;
export let rel: AnchorHTMLAttributes['rel'] = undefined;
// Alias for the `class` attribute since `class` is a reserved TS keyword
export let className: string | undefined = undefined;
export let underline = false;

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

const dispatch = createEventDispatcher();

function handleClick(event: MouseEvent) {
	dispatch('click', event);
}

$: baseClasses = 'hover:underline text-primary dark:text-white';
$: ({ class: restClass, ...restProps } = $$restProps);
$: classes =
	[baseClasses, restClass, className, underline ? 'underline' : undefined]
		.filter(Boolean)
		.join(' ') || undefined;
$: resolvedRel = target === '_blank' && !rel ? 'noreferrer noopener' : rel;
</script>

<a href={href} target={target} rel={resolvedRel} class={classes} {...restProps}
		on:click={handleClick}
>
	<slot />
</a>

<script lang="ts">
import { createEventDispatcher } from 'svelte';
import type { HTMLInputAttributes } from 'svelte/elements';

interface $$Slots {
	leading: Record<string, never>;
	trailing: Record<string, never>;
}

export let value: HTMLInputAttributes['value'] = undefined;
export let placeholder: HTMLInputAttributes['placeholder'] = undefined;
export let disabled: HTMLInputAttributes['disabled'] = undefined;
export let className = '';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};
const dispatch = createEventDispatcher<{ input: Event; keydown: KeyboardEvent }>();

$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = ['search-field', restClass, className].filter(Boolean).join(' ');
</script>

<div class={classes}>
	<slot name="leading" />
	<input
		type="text"
		bind:value
		{placeholder}
		{disabled}
		{...restProps}
		on:input={(event) => dispatch('input', event)}
		on:keydown={(event) => dispatch('keydown', event)}
	/>
	<slot name="trailing" />
</div>

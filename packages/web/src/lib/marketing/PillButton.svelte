<script lang="ts">
export let href = '';
export let kind: 'primary' | 'secondary' = 'primary';
export let size: 'md' | 'lg' = 'md';
export let disabled = false;
export let label = '';

$: classes = [
	'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full font-bold no-underline transition duration-150 ease-out hover:-translate-y-px hover:no-underline',
	size === 'lg' ? 'h-11 px-5 text-sm' : 'h-9 px-4 text-[0.82rem]',
	kind === 'primary'
		? 'border border-black/40 bg-black !text-white dark:border-white/40 dark:bg-white dark:!text-black'
		: 'border border-black/10 bg-black/5 !text-black dark:border-white/15 dark:bg-white/10 dark:!text-white',
	disabled ? 'cursor-default opacity-70 hover:translate-y-0' : '',
	'[&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:fill-current',
]
	.filter(Boolean)
	.join(' ');
</script>

{#if disabled}
	<span class={classes} role="link" aria-disabled="true" aria-label={label || undefined}>
		<slot name="icon" />
		<slot />
	</span>
{:else}
	<a class={classes} {href} aria-label={label || undefined}>
		<slot name="icon" />
		<slot />
	</a>
{/if}

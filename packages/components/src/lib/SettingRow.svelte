<script lang="ts">
export let top = false;
export let className = '';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

$: ({ class: restClass, ...restProps } = $$restProps);
const baseClasses = 'flex gap-6 [&>div:first-child]:min-w-0 [&>div:first-child]:flex-1';

$: classes = [
	'row',
	baseClasses,
	top && 'top',
	top ? 'items-start' : 'items-center',
	restClass,
	className,
]
	.filter(Boolean)
	.join(' ');
</script>

<div class={classes} {...restProps}>
	<div>
		<slot />
	</div>
	<slot name="control" />
</div>

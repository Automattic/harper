<script lang="ts">
export let className: string | undefined = undefined;

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

const baseClasses =
	'rounded-lg px-4 py-3 shadow-lg backdrop-blur border border-cream-100 dark:border-cream-700';

$: ({ class: restClass, ...restProps } = $$restProps);
$: classes = [baseClasses, restClass, className].filter(Boolean).join(' ');
</script>

<div class={classes} {...restProps}>
	<slot />
</div>

<script lang="ts">
export let className = '';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};

$: ({ class: restClass, ...restProps } = $$restProps);
const baseClasses =
	'overflow-hidden rounded-[10px] border-[0.5px] border-[var(--settings-line-strong,rgb(28_26_22_/_14%))] bg-[#fff]';

$: classes = ['list-card', baseClasses, restClass, className].filter(Boolean).join(' ');
</script>

<div class={classes} {...restProps}>
	<slot />
</div>

<script lang="ts">
export let title: string;
export let open = false;
export let className = '';

const baseClasses =
	'group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900';

$: detailsClass = `${baseClasses} ${className}`.trim();
</script>

<details class={detailsClass} {open}>
	<summary class="cursor-pointer font-semibold marker:text-neutral-400">{title}</summary>
	<div class="mt-3">
		<slot />
	</div>
</details>

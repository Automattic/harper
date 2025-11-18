<script lang="ts">
export let authorName: string;
export let authorSubtitle: string;
export let testimonial: string;
</script>

<article class="flex h-full flex-col justify-between gap-6 rounded-xl border border-neutral-200 bg-white p-6 shadow-sm break-inside-avoid dark:border-neutral-800 dark:bg-neutral-900">
	<p class="text-base leading-relaxed text-neutral-700 dark:text-neutral-200">
		{testimonial}
	</p>
	<footer class="flex flex-col gap-1">
		<span class="font-semibold text-neutral-900 dark:text-neutral-50">{authorName}</span>
		<span class="text-sm text-neutral-600 dark:text-neutral-400">{authorSubtitle}</span>
	</footer>
</article>

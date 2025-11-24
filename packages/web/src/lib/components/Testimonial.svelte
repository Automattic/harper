<script lang="ts">
import { Card } from 'components';

export let authorName: string;
export let authorSubtitle: string;
export let testimonial: string;
</script>

<Card class="flex h-full flex-col justify-between gap-6">
	<p class="text-base text-neutral-700 dark:text-neutral-200">
		{testimonial}
	</p>
	<footer class="flex flex-col gap-1">
		<span class="font-semibold text-neutral-900 dark:text-neutral-50">{authorName}</span>
		<span class="text-sm text-neutral-600 dark:text-neutral-400">{authorSubtitle}</span>
	</footer>
</Card>

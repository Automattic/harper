<script lang="ts">
import HarperMark from './HarperMark.svelte';

export let word: string;
export let fix: string;
</script>

<div
	class='absolute right-3 bottom-2.5 inline-flex items-center gap-1.5 rounded-lg border border-black/10 bg-white px-2 py-1 text-[0.68rem] text-black shadow-[0_6px_16px_rgba(0,0,0,0.16)] dark:border-white/10 dark:bg-black dark:text-white'
>
	<HarperMark size={11} />
	<del class='text-black/50 [font-family:"JetBrains_Mono",monospace] dark:text-white/50'>{word}</del>
	<span class="text-black/50 dark:text-white/50">→</span>
	<strong class='[font-family:"JetBrains_Mono",monospace]'>{fix}</strong>
</div>

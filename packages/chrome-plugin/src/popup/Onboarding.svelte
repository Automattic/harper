<script lang="ts">
import { Button } from 'components';

let { onConfirm }: { onConfirm: () => void } = $props();

let isConfirming = $state(false);

const steps = [
	'Start typing in any large text box — emails, docs, blog posts, you name it.',
	'Keep writing — Harper quietly highlights potential hiccups as you go.',
	'Click a highlight to open focused, context-aware suggestions.',
];

function handleStart() {
	if (isConfirming) return;
	isConfirming = true;
	// Allow button press state/animation to register briefly before transitioning views
	setTimeout(() => {
		onConfirm();
	}, 180);
}
</script>

<main class="flex h-full flex-col justify-between px-5 py-4 select-none">
	<div class="space-y-4">
		<div>
			<h2 class="text-base font-semibold dark:text-white">
				Welcome! Let’s see Harper in action:
			</h2>
			<p class="mt-0.5 text-xs text-gray-500 dark:text-slate-400">
				Three quick tips to get the most out of your writing assistant.
			</p>
		</div>

		<ul class="space-y-3.5 pt-1">
			{#each steps as line, i}
				<li class="flex items-start gap-3">
					<span
						class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md  bg-primary/15 text-xs font-bold text-primary ring-1 ring-primary/30 dark:bg-primary/20 dark:text-primary dark:ring-primary/40"
					>
						{i + 1}
					</span>
					<p class="text-sm leading-relaxed text-gray-700 dark:text-slate-300">
						{line}
					</p>
				</li>
			{/each}
		</ul>
	</div>

	<div class="pt-3">
		<Button
			color="primary"
			class="h-9 w-full text-xs font-semibold bg-primary! text-black! hover:bg-primary/85!"
			disabled={isConfirming}
			on:click={handleStart}
		>
			{isConfirming ? 'Getting started...' : "Let's start writing"}
		</Button>
	</div>
</main>
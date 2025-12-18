<script lang="ts">
import { Badge, Button } from 'components';

let { onConfirm }: { onConfirm: () => void } = $props();

const steps = [
	'Start typing in any large text box — emails, docs, blog posts, you name it.',
	'Keep writing — Harper quietly highlights potential hiccups as you go.',
	'Click a highlight to open focused, context‑aware suggestions.',
];
</script>

<main class="p-6 space-y-6">
  <h2 class="text-base font-semibold">
    Welcome! Let’s see Harper in action:
  </h2>

  <ul class="space-y-6">
    {#each steps as line, i}
      <li class="flex gap-4 items-start">
        <Badge class="w-8 h-8 bg-primary-300 text-primary-900 font-semibold flex items-center justify-center rounded-lg dark:bg-slate-900 dark:text-slate-100">
          {i + 1}
        </Badge>
        <p class="text-sm text-gray-700 dark:text-slate-300">{line}</p>
      </li>
    {/each}
  </ul>

  <Button color="primary"  class="w-full h-10" on:click={onConfirm}>
    Let's start writing
  </Button>
</main>

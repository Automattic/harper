<script lang="ts">
import { lintKindColor, type UnpackedLint, type UnpackedSuggestion } from 'lint-framework';
import { slide } from 'svelte/transition';

export let lint: UnpackedLint;
export let open = false;
export let onToggle: () => void;
export let onApply: (s: UnpackedSuggestion) => void;
export let snippet: {
	prefix: string;
	problem: string;
	suffix: string;
	prefixEllipsis: boolean;
	suffixEllipsis: boolean;
};

function suggestionText(s: UnpackedSuggestion): string {
	return s.replacement_text !== '' ? s.replacement_text : String(s.kind);
}
</script>

<div class="rounded-lg border border-gray-300 dark:border-gray-700 shadow-sm bg-white dark:bg-[#0d1117]">
  <div
    role="button"
    tabindex="0"
    class="flex items-center justify-between p-3 cursor-pointer select-none"
    aria-expanded={open}
    on:click={() => onToggle?.()}
    on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), onToggle?.())}
  >
    <div
      class="text-sm font-semibold pb-1"
      style={`border-bottom: 2px solid ${lintKindColor(lint.lint_kind)}`}
    >
      {lint.lint_kind_pretty}
    </div>
    <svg
      class={`ml-3 h-4 w-4 transform transition-transform duration-200 ${open ? 'rotate-180' : ''}`}
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
    >
      <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.127l3.71-3.896a.75.75 0 111.08 1.04l-4.243 4.46a.75.75 0 01-1.08 0L5.25 8.27a.75.75 0 01-.02-1.06z" clip-rule="evenodd" />
    </svg>
  </div>
  {#if open}
    <div class="px-3 pb-3" in:slide={{ duration: 150 }} out:slide={{ duration: 150 }}>
      <div class="text-sm text-gray-700 dark:text-gray-300 mb-2 break-words">
        {@html lint.message_html}
      </div>
      <div class="text-xs font-mono mb-2 p-2 rounded border border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-[#0b0f14] text-gray-800 dark:text-gray-200 leading-snug">
        <span class="text-gray-500">{snippet.prefixEllipsis ? '…' : ''}{snippet.prefix}</span>
        <span class="px-0.5 rounded bg-yellow-200 text-black dark:bg-yellow-800 dark:text-yellow-100">{snippet.problem}</span>
        <span class="text-gray-500">{snippet.suffix}{snippet.suffixEllipsis ? '…' : ''}</span>
      </div>
      {#if lint.suggestions && lint.suggestions.length > 0}
        <div class="flex flex-wrap gap-2 justify-end">
          {#each lint.suggestions as s}
            <button
              class="inline-flex items-center justify-center rounded-md px-2 py-1 text-xs font-semibold"
              style="background:#2DA44E;color:#FFFFFF"
              title={`Replace with \"${suggestionText(s)}\"`}
              on:click={() => onApply?.(s)}
            >
              {suggestionText(s)}
            </button>
          {/each}
        </div>
      {:else}
        <div class="text-xs text-gray-400">No suggestions available.</div>
      {/if}
    </div>
  {/if}
</div>

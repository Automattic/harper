<script lang="ts">
import { Button, Card } from 'flowbite-svelte';
import { SuggestionKind, type WorkerLinter } from 'harper.js';
import {
	applySuggestion,
	LintFramework,
	type UnpackedLint,
	type UnpackedSuggestion,
	unpackLint,
} from 'lint-framework';
import demo from '../../../../demo.md?raw';

export let content = demo.trim();

let editor: HTMLTextAreaElement | null;
let linter: WorkerLinter;

// Live list of lints from the framework's lint callback
let lints: UnpackedLint[] = [];

let lfw = new LintFramework(async (text) => {
	// Guard until the linter is ready
	if (!linter) return [];

	const raw = await linter.lint(text);
	// The framework expects "unpacked" lints with plain fields
	const unpacked = await Promise.all(
		raw.map((lint) => unpackLint(window.location.hostname, lint as any, linter as any)),
	);

	lints = unpacked;

	return unpacked;
}, {});

(async () => {
	let { WorkerLinter, binary } = await import('harper.js');
	linter = new WorkerLinter({ binary });

	await linter.setup();
})();

$: if (editor != null) {
	lfw.addTarget(editor);
}

function suggestionLabel(s: UnpackedSuggestion): string {
	switch (s.kind) {
		case SuggestionKind.Remove:
			return 'Remove';
		case SuggestionKind.Replace:
			return s.replacement_text ? `Replace with "${s.replacement_text}"` : 'Replace';
		case SuggestionKind.InsertAfter:
			return s.replacement_text ? `Insert "${s.replacement_text}"` : 'Insert';
	}
}

function applySug(lint: UnpackedLint, s: UnpackedSuggestion) {
	content = applySuggestion(content, lint.span, s);
	// Trigger re-lint and rerender after programmatic change
	lfw.update();
}
</script>

<div class="flex flex-row h-full max-w-full">
	<Card class="flex-1 h-full p-5 z-10 max-w-full text-lg mr-5">
		<textarea
			bind:this={editor}
			class="w-full m-0 rounded-none p-0 z-0 bg-transparent h-full border-none text-lg resize-none focus:border-0"
			bind:value={content}
		></textarea>
	</Card>

	<Card class="hidden md:flex md:flex-col md:w-1/3 h-full p-5 z-10">
		<div class="text-base font-semibold mb-3">Lints</div>
		<div class="flex-1 overflow-y-auto pr-1">
			{#if lints.length === 0}
				<p class="text-sm text-gray-500">No lints yet.</p>
			{:else}
				<div class="space-y-3">
					{#each lints as lint}
						<Card class="p-3">
							<div class="text-sm font-medium">{lint.lint_kind_pretty}</div>
							<div class="text-xs text-gray-600 mb-2 break-words">{lint.problem_text}</div>
							{#if lint.suggestions && lint.suggestions.length > 0}
								<div class="flex flex-wrap gap-2">
									{#each lint.suggestions as s}
										<Button size="xs" on:click={() => applySug(lint, s)}>{suggestionLabel(s)}</Button>
									{/each}
								</div>
							{:else}
								<div class="text-xs text-gray-400">No suggestions available.</div>
							{/if}
						</Card>
					{/each}
				</div>
			{/if}
		</div>
	</Card>
</div>

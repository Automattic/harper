<script lang="ts">
import { Card } from 'flowbite-svelte';
import { type WorkerLinter } from 'harper.js';
import {
	applySuggestion,
	LintFramework,
	lintKindColor,
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
		raw.map((lint) => unpackLint(window.location.hostname, lint, linter)),
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

function suggestionText(s: UnpackedSuggestion): string {
	return s.replacement_text !== '' ? s.replacement_text : String(s.kind);
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
		<div class="text-base font-semibold mb-3">Problems</div>
		<div class="flex-1 overflow-y-auto pr-1">
			{#if lints.length === 0}
				<p class="text-sm text-gray-500">No lints yet.</p>
			{:else}
				<div class="space-y-3">
					{#each lints as lint}
						<div class="rounded-lg border border-gray-300 dark:border-gray-700 shadow-sm p-3 bg-white dark:bg-[#0d1117]">
							<div class="text-sm font-semibold pb-1 mb-2" style={`border-bottom: 2px solid ${lintKindColor(lint.lint_kind)}`}>{lint.lint_kind_pretty}</div>
							<div class="text-sm text-gray-700 dark:text-gray-300 mb-2 break-words">
								{@html lint.message_html}
							</div>
							{#if lint.suggestions && lint.suggestions.length > 0}
								<div class="flex flex-wrap gap-2 justify-end">
									{#each lint.suggestions as s}
										<button
											class="inline-flex items-center justify-center rounded-md px-2 py-1 text-xs font-semibold"
											style="background:#2DA44E;color:#FFFFFF"
											title={`Replace with \"${suggestionText(s)}\"`}
											on:click={() => applySug(lint, s)}
										>
											{suggestionText(s)}
										</button>
									{/each}
								</div>
							{:else}
								<div class="text-xs text-gray-400">No suggestions available.</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</Card>
</div>

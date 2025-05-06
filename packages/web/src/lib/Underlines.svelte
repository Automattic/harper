<script lang="ts">
// This is some of the shittiest code I've ever written.
// It is quite hard to look at.
// Someday, I'll return to it and spruce it up.
// For now, it works.

import lintKindColor from '$lib/lintKindColor';
import type { Lint, WorkerLinter } from 'harper.js';

export let content: string;
export let focusLintIndex: number | undefined;

import { quintOut } from 'svelte/easing';

let loadTime = Date.now();

function slideUnderline(_node: HTMLElement) {
	return {
		duration: 300,
		css: (t: number) => {
			if (Date.now() - loadTime > 2000) {
				t = 1;
			}

			return `
        transform: scaleX(${t});
        transform-origin: left;
      `;
		},
		easing: quintOut,
	};
}

let lints: [Lint, number][] = [];
let lintHighlights: HTMLSpanElement[] = [];
let linter: WorkerLinter;

(async () => {
	let { WorkerLinter, binary } = await import('harper.js');

	linter = new WorkerLinter({ binary });

	await linter.setup();
})();

$: linter?.lint(content).then((newLints) => {
	lints = newLints
		.map<[Lint, number]>((lint, index) => [lint, index])
		.toSorted(([a], [b]) => a.span().start - b.span().end);
});
$: if (focusLintIndex != null && lintHighlights[focusLintIndex] != null)
	lintHighlights[focusLintIndex].scrollIntoView({
		behavior: 'smooth',
		block: 'nearest',
		inline: 'nearest',
	});

function reOrgString(text: string): (string | undefined)[] {
	let output: (string | undefined)[] = [];

	for (let chunk of text.replaceAll(' ', '\u00A0').split('\n')) {
		if (output.length > 0) {
			output.push(undefined);
		}
		output.push(chunk);
	}

	return output;
}

type UnderlineDetails = {
	focused: boolean;
	content: string;
	index: number;
	color: string;
	context: string;
};

type UnderlineToken = string | null | undefined | UnderlineDetails;

function processString(lintMap: [Lint, number][], focusLintIndex?: number) {
	let results: UnderlineToken[] = lintMap.flatMap(([lint, lintIndex], index, arr) => {
		let prevStart = 0;
		let prev = arr[index - 1];

		if (prev != null) {
			prevStart = prev[0].span().end;
		}

		let prevEnd = lint.span().start;

		let prevContent = [];

		if (prevStart !== prevEnd) {
			prevContent.push(...reOrgString(content.substring(prevStart, prevEnd)));
		}

		let lintContent: UnderlineDetails = {
			focused: lintIndex === focusLintIndex,
			index: lintIndex,
			content: lint.get_problem_text().replaceAll(' ', '\u00A0'),
			color: lintKindColor(lint.lint_kind()),
			context: prevContent[prevContent.length - 1] ?? '',
		};

		return [...prevContent, lintContent];
	});

	let lastLint = lints.at(-1);

	let finalChunk: string;

	if (lastLint != null) {
		finalChunk = content.substring(lastLint[0].span().end);
	} else {
		finalChunk = content;
	}

	results.push(...reOrgString(finalChunk));

	return results;
}

// string | [string, string, string, index] | null
$: modified = processString(lints, focusLintIndex);
</script>

<div class="grid">
	<div class="p-0 m-0 text-nowrap indent-0" style="grid-row: 1; grid-column: 1">
		{#each modified as chunk}
			{#if chunk == null}
				<br />
			{:else if typeof chunk == 'string'}
				<span class="whitespace-pre !text-transparent">{chunk}</span>
			{:else}
				<span class="pointer-events-auto">
					<button
						class={`underlinespecial transition-all rounded-sm ${chunk.focused ? 'animate-after-bigbounce text-white' : 'text-transparent'}`}
						bind:this={lintHighlights[chunk.index]}
						in:slideUnderline
						on:click={() =>
							chunk != null && typeof chunk == 'object' && (focusLintIndex = chunk.index)}
						style={`--line-color: ${chunk.color}; --line-width: ${chunk.focused ? '4px' : '2px'}; --bg-color: ${chunk.focused ? '#dbafb3' : 'transparent'};`}
					>
						{chunk.content}
					</button>
				</span>
			{/if}
		{/each}
	</div>
</div>

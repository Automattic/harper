<script lang="ts">
import CheckMark from '$lib/CheckMark.svelte';
import Underlines from '$lib/Underlines.svelte';
import { Button, Card } from 'flowbite-svelte';
import { type Lint, SuggestionKind, type WorkerLinter } from 'harper.js';
import { fly } from 'svelte/transition';
import demo from '../../../../demo.md?raw';
import lintKindColor from './lintKindColor';

export let content = demo;

let lints: Lint[] = [];
let lintCards: HTMLButtonElement[] = [];
let focused: number | undefined;
let editor: HTMLTextAreaElement | null;
let linter: WorkerLinter;

(async () => {
	let { WorkerLinter, binary } = await import('harper.js');
	linter = new WorkerLinter({ binary });

	await linter.setup();
})();

let w: number | undefined;

$: linter?.lint(content).then((newLints) => {
	lints = newLints;
});
$: boxHeight = calcHeight(content);
$: if (focused != null && lintCards[focused])
	lintCards[focused].scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
$: if (focused != null && focused >= lints.length) focused = undefined;

$: if (editor != null && focused != null) {
	let lint = lints[focused % lints.length];
	if (lint != null) {
		let p = lint.span().end;
		editor.selectionStart = p;
		editor.selectionEnd = p;
	}
}

function calcHeight(boxContent: string): number {
	let numberOfLineBreaks = (boxContent.match(/\n/g) || []).length;
	let newHeight = 20 + numberOfLineBreaks * 30 + 12 + 2;
	return newHeight;
}

// Whether to display a smallar variant of the editor
$: small = (w ?? 1024) < 1024;
$: superSmall = (w ?? 1024) < 550;
</script>

<div class={`flex w-full h-full p-5 ${small ? 'flex-col' : 'flex-row'}`} bind:clientWidth={w}>
	<Card
		class="flex-grow h-full p-5 grid z-10 max-w-full text-lg overflow-auto mr-5"
		on:click={() => editor && editor.focus()}
	>
		<textarea
			bind:this={editor}
			class="w-full text-nowrap m-0 rounded-none p-0 z-0 bg-transparent overflow-hidden border-none text-lg resize-none focus:border-0"
			spellcheck="false"
			data-enable-grammarly="false"
			style={`grid-row: 1; grid-column: 1; height: ${boxHeight}px`}
			on:keydown={() => (focused = undefined)}
			bind:value={content}
		></textarea>
		<div class="m-0 p-0 z-10 pointer-events-none" style="grid-row: 1; grid-column: 1">
			<Underlines {content} bind:focusLintIndex={focused} />
		</div>
	</Card>
	<Card class={`flex-none basis-[400px] max-h-full p-1 ${small ? 'hidden' : 'flex'}`}>
		<h2 class="text-2xl font-bold m-2">Suggestions</h2>
		<div class="flex flex-col overflow-y-auto overflow-x-hidden m-0 p-0 h-full">
			{#if lints.length == 0}
				<div class="w-full h-full flex flex-row text-center justify-center items-center" in:fly>
					<p class="dark:white font-bold text-lg">Looks good to me</p>
					<CheckMark />
				</div>
			{/if}

			{#each lints as lint, i}
				<button
					class="block max-w-sm p-3 bg-white dark:bg-gray-800 border border-gray-200 rounded-lg shadow m-1 hover:translate-x-1 transition-all"
					on:click={() => (focused = i)}
					bind:this={lintCards[i]}
				>
					<div class={`pl-2`} style={`border-left: 4px solid ${lintKindColor(lint.lint_kind())}`}>
						<div class="flex flex-row">
							<h3 class="font-bold text-base p-0">
								{lint.lint_kind_pretty()} - “<span class="italic">
									{lint.get_problem_text()}
								</span>”
							</h3>
						</div>
						<div
							class="transition-all overflow-hidden flex flex-col justify-evenly"
							style={`height: ${focused === i ? `calc(55px * ${lint.suggestion_count() + 1})` : '0px'}`}
						>
							<p style="height: 50px" class="text-left text-sm p-0">{@html lint.message_html().replaceAll('<p>', "").replaceAll('<p />', "")}</p>
							{#each lint.suggestions() as suggestion}
								<div class="w-full p-[4px]">
									<Button
										class="w-full"
										style="height: 40px; margin: 5px 0px;"
										on:click={() =>
											linter
												.applySuggestion(content, lint, suggestion)
												.then((edited) => (content = edited))}
									>
										{#if suggestion.kind() == SuggestionKind.Remove}
											Remove "{lint.get_problem_text()}"
										{:else if suggestion.kind() == SuggestionKind.Replace}
											Replace "{lint.get_problem_text()}" with "{suggestion.get_replacement_text()}"
										{:else}
											Insert "{suggestion.get_replacement_text()}" after "{lint.get_problem_text()}"
										{/if}
									</Button>
								</div>
							{/each}
						</div>
					</div>
				</button>
			{/each}
		</div>
	</Card>
	{#if focused != null}
		<Card
			class={`max-w-full w-full ${superSmall ? 'justify-center' : 'justify-between'} flex-row ${small ? '' : 'hidden'}`}
		>
			<div class={superSmall ? 'hidden' : ''}>
				<h1 class={`font-bold p-0 text-base`}>{lints[focused].lint_kind_pretty()}</h1>
				<p class={`p-0 text-sm`}>{@html lints[focused].message_html().replaceAll('<p>', "").replaceAll('<p />', "")}</p>
			</div>
			<div class="flex flex-row">
				{#each lints[focused].suggestions() as suggestion}
					<div class="p-[4px]">
						<Button
							class="w-full"
							style="height: 40px; margin: 5px 0px;"
							on:click={() =>
								focused != null &&
								linter
									.applySuggestion(content, lints[focused], suggestion)
									.then((edited) => (content = edited))}
						>
							{#if suggestion.kind() == SuggestionKind.Remove}
								Remove
							{:else}
								"{suggestion.get_replacement_text()}"
							{/if}
						</Button>
					</div>
				{/each}
			</div>
		</Card>
	{/if}
</div>

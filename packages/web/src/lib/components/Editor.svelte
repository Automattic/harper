<script lang="ts">
import { Card } from 'flowbite-svelte';
import { type WorkerLinter } from 'harper.js';
import {
	applySuggestion,
	type IgnorableLintBox,
	type IgnorableLintBox,
	LintFramework,
	type UnpackedLint,
	type UnpackedLintGroups,
	type UnpackedSuggestion,
	unpackLint,
} from 'lint-framework';
import LintSidebar from '$lib/components/LintSidebar.svelte';
import demo from '../../../../../demo.md?raw';

export let content = demo.trim();

let editor: HTMLDivElement | null;
let linter: WorkerLinter;

// Live list of lints from the framework's lint callback
let lintBoxes: IgnorableLintBox[] = [];
let lfw = new LintFramework(
	async (text) => {
		if (!linter) return {};

		const raw = await linter.organizedLints(text);
		// The framework expects grouped lints keyed by source
		const entries = await Promise.all(
			Object.entries(raw).map(async ([source, lintGroup]) => {
				const unpacked = await Promise.all(lintGroup.map((lint) => unpackLint(text, lint, linter)));
				return [source, unpacked] as const;
			}),
		);

		const grouped: UnpackedLintGroups = Object.fromEntries(entries);

		lintBoxes = lfw.getLastIgnorableLintBoxes();

		return grouped;
	},
	{
		ignoreLint: async (hash: string) => {
			if (!linter) return;
			try {
				await linter.ignoreLintHash(BigInt(hash));
				console.log(`Ignored ${hash}`);
				// Re-run linting to hide ignored lint immediately
				lfw.update();
			} catch (e) {
				console.error('Failed to ignore lint', e);
			}
		},
	},
);

(async () => {
	let { WorkerLinter, binary } = await import('harper.js');
	linter = new WorkerLinter({ binary });

	await linter.setup();
})();

let quill: any;

async function updateLintFrameworkElements() {
	if (editor == null) {
		return;
	}

	if (quill == null) {
		let { default: Quill } = await import('quill');
		quill = new Quill(editor, {});
	}

	for (let el of editor.getElementsByTagName('p')) {
		lfw.addTarget(el);
	}
}

$: if (editor != null) {
	let mo = new MutationObserver(updateLintFrameworkElements);
	mo.observe(editor, { childList: true, subtree: true });
	updateLintFrameworkElements();
}

function jumpTo(lintBox: IgnorableLintBox) {

}

async function ignoreAll() {
	if (!linter || lintBoxes.length === 0) return;
	try {
		const hashes = Array.from(new Set(lintBoxes.map((b) => b.lint.context_hash)));
		await Promise.all(hashes.map((h) => linter.ignoreLintHash(BigInt(h))));
		// Refresh to hide ignored lints immediately
		lfw.update();
	} catch (e) {
		console.error('Failed to ignore all lints', e);
	}
}
</script>

<div class="flex flex-row h-full max-w-full">
	<Card class="flex-1 h-full p-5 z-10 max-w-full text-lg mr-5">
    <div bind:this={editor} class="w-full m-0 rounded-none p-0 z-0 bg-transparent h-full border-none text-lg resize-none focus:border-0">
    {@html content.replace(/\n\n/g, '<br>')}
    </div>
	</Card>

	<LintSidebar
		lintBoxes={lintBoxes}
		content={content}
		focusLint={jumpTo}
		ignoreAll={ignoreAll}
	/>
</div>

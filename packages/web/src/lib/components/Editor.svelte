<script lang="ts">
import { Card } from 'components';
import { type WorkerLinter } from 'harper.js';
import {
	type IgnorableLintBox,
	LintFramework,
	type UnpackedLintGroups,
	unpackLint,
} from 'lint-framework';
import LintSidebar from '$lib/components/LintSidebar.svelte';
import demo from '../../../../../demo.md?raw';

export let content = demo.trim();
export let onReady: () => void = () => null;

let editor: HTMLDivElement | null;
let linter: WorkerLinter;
let quill: any;
let lintBoxes: IgnorableLintBox[] = [];

$: if (linter != null && quill != null) {
	onReady();
}

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
	let newLinter = new WorkerLinter({ binary });

	newLinter.setup();
	await newLinter.lint(content);
	linter = newLinter;
})();

async function updateLintFrameworkElements() {
	if (editor == null) {
		return;
	}

	if (quill == null) {
		let { default: Quill } = await import('quill');
		quill = new Quill(editor, {});
		const container = quill.container ?? quill.root?.parentElement;
		container?.classList.add('h-full', 'min-h-0');

		quill.root?.classList.add('flex', 'flex-col', 'h-full', 'min-h-0', 'outline-transparent');
		quill.root?.setAttribute('data-enable-grammarly', 'false');
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
	if (typeof window === 'undefined') {
		return;
	}

	const range = lintBox.range;
	if (!range) {
		return;
	}

	try {
		const rect = range.getBoundingClientRect();

		const selection = window.getSelection();
		if (selection) {
			selection.removeAllRanges();
			selection.addRange(range.cloneRange());
		}

		const margin = Math.max(10, window.innerHeight * 0.2);
		const target = Math.max(0, window.scrollY + rect.top - margin);
		window.scrollTo({ top: target, behavior: 'smooth' });
	} catch (error) {
		console.error('Failed to jump to lint', error);
	}
}
</script>

<div class="flex flex-row h-full w-full">
	<Card class="flex-1 h-full p-5 z-10 max-w-full text-lg mr-5">
    <div bind:this={editor} spellcheck="false">
    {@html content.replace(/\n\n/g, '<br>')}
    </div>
	</Card>

	<LintSidebar
		lintBoxes={lintBoxes}
		focusLint={jumpTo}
	/>
</div>

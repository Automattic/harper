<script lang="ts">
import type { Lint, Linter } from 'harper.js';
import {
	type IgnorableLintBox,
	LintFramework,
	type UnpackedLintGroups,
	unpackLint,
} from 'lint-framework';
import { tick } from 'svelte';
import { fade } from 'svelte/transition';
import {
	type EditorFontFamily,
	type EditorFontSize,
	fontStackFor,
	normalizeFontFamily,
	normalizeFontSize,
} from './editorDisplay.js';
import LintSidebar from './LintSidebar.svelte';
import StatusBar from './StatusBar.svelte';

export let content = '';
export let linter: Linter;
export let onReady: () => void = () => null;
export let defaultFontFamily: EditorFontFamily = 'sans';
export let defaultFontSize: EditorFontSize = 'default';
export let onChange: (text: string) => void = () => {};

let editor: HTMLDivElement | null;
let linterVersion = 0;
let quill: any;
let lintBoxes: IgnorableLintBox[] = [];
let activeLintId: string | null = null;
let documentText = content;
let fontFamily = normalizeFontFamily(defaultFontFamily);
let fontSize = normalizeFontSize(defaultFontSize);
let lastExternalContent = content;
let readySent = false;
let restoreButtonTimeout: ReturnType<typeof setTimeout> | null = null;
let restoreButtonVisible = false;
let sidebarVisible = true;
let syncTimeout: ReturnType<typeof setTimeout> | null = null;

const sidebarTransitionDuration = 250;
const restoreButtonDelay = sidebarTransitionDuration + 40;

$: fontStack = fontStackFor(fontFamily);
$: editorStyle =
	`--harper-editor-font-family: ${fontStack};` +
	(fontSize === 'default' ? '' : ` --harper-editor-font-size: ${fontSize}px;`);

$: if (linter != null && quill != null) {
	if (!readySent) {
		readySent = true;
		onReady();
	}
}

let lfw = new LintFramework(
	async (text) => {
		const raw = await linter.organizedLints(text);
		// The framework expects grouped lints keyed by source
		const entries = await Promise.all(
			Object.entries(raw).map(async ([source, lintGroup]: [string, Lint[]]) => {
				const unpacked = await Promise.all(lintGroup.map((lint) => unpackLint(text, lint, linter)));
				return [source, unpacked] as const;
			}),
		);

		const grouped: UnpackedLintGroups = Object.fromEntries(entries);

		scheduleLintBoxSync();

		return grouped;
	},
	{
		ignoreLint: async (hash: string) => {
			try {
				await linter.ignoreLintHash(BigInt(hash));
				console.log(`Ignored ${hash}`);
				// Re-run linting to hide ignored lint immediately
				lfw.update();
				scheduleLintBoxSync();
			} catch (e) {
				console.error('Failed to ignore lint', e);
			}
		},
	},
);

$: {
	const version = ++linterVersion;
	const activeLinter = linter;

	lintBoxes = [];

	void (async () => {
		try {
			await activeLinter.setup();
			await activeLinter.lint(content);
		} catch (error) {
			console.error('Failed to initialize linter', error);
		}

		if (version !== linterVersion) {
			return;
		}

		if (editor != null) {
			lfw.update();
		}
	})();
}

async function updateLintFrameworkElements() {
	if (editor == null) {
		return;
	}

	if (quill == null) {
		let { default: Quill } = await import('quill');
		quill = new Quill(editor, {});
		const container = quill.container ?? quill.root?.parentElement;
		container?.classList.add('harper-editor-quill-container');

		quill.root?.classList.add('harper-editor-surface');
		quill.root?.setAttribute('data-enable-grammarly', 'false');
		quill.root?.setAttribute('spellcheck', 'false');
		setQuillText(content, false);
		quill.on('text-change', () => {
			syncDocumentText(true);
			scheduleLintFrameworkUpdate();
		});
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

$: if (quill != null && content !== lastExternalContent) {
	lastExternalContent = content;
	if (content !== documentText) {
		setQuillText(content, false);
		scheduleLintFrameworkUpdate();
	}
}

function jumpTo(lintBox: IgnorableLintBox) {
	if (typeof window === 'undefined') {
		return;
	}

	activeLintId = lintBox.lint.context_hash;

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

// Quill always keeps a trailing document newline; callers expect plain text.
function normalizeQuillText(text: string): string {
	return text.endsWith('\n') ? text.slice(0, -1) : text;
}

function setQuillText(text: string, notify: boolean) {
	if (quill == null) {
		documentText = text;
		return;
	}

	const source = notify ? 'user' : 'silent';
	quill.setText(text, source);
	syncDocumentText(notify);
}

// Keep the public text state and `onChange` callback in sync with Quill's document.
function syncDocumentText(notify: boolean) {
	if (quill == null) {
		return;
	}

	const next = normalizeQuillText(quill.getText());
	if (next === documentText) {
		return;
	}

	documentText = next;
	if (notify) {
		onChange(next);
	}
}

// The lint framework owns highlight DOM, so copy its latest boxes into Svelte state.
function syncLintBoxes() {
	lintBoxes = [...lfw.getLastIgnorableLintBoxes()];
	if (activeLintId != null && !lintBoxes.some((lintBox) => lintBox.lint.context_hash === activeLintId)) {
		activeLintId = null;
	}
}

// Lint decorations settle across layout frames; the timeout catches slower browser updates.
function scheduleLintBoxSync() {
	requestAnimationFrame(() => {
		requestAnimationFrame(syncLintBoxes);
	});

	if (syncTimeout != null) {
		clearTimeout(syncTimeout);
	}

	syncTimeout = setTimeout(syncLintBoxes, 150);
}

// Refresh target elements, ask the framework to lint, then mirror its current boxes.
function scheduleLintFrameworkUpdate() {
	updateLintFrameworkElements();
	lfw.update();
	scheduleLintBoxSync();
}

// Suggestions and ignores mutate the document/lint state outside Quill's text-change path.
function handleProblemAction() {
	syncDocumentText(true);
	scheduleLintFrameworkUpdate();
}

async function ignoreAllProblems() {
	syncDocumentText(false);

	const text = documentText;
	const activeLinter = linter;
	const groupedLints = await activeLinter.organizedLints(text, { dedup: false });
	const lints = Object.values(groupedLints).flat();

	if (lints.length === 0) {
		return;
	}

	await activeLinter.ignoreLints(text, lints);
}

async function showSidebar() {
	if (restoreButtonTimeout != null) {
		clearTimeout(restoreButtonTimeout);
		restoreButtonTimeout = null;
	}

	restoreButtonVisible = false;
	await tick();
	sidebarVisible = true;
}

function hideSidebar() {
	sidebarVisible = false;
	restoreButtonVisible = false;

	if (restoreButtonTimeout != null) {
		clearTimeout(restoreButtonTimeout);
	}

	restoreButtonTimeout = setTimeout(() => {
		if (!sidebarVisible) {
			restoreButtonVisible = true;
		}
		restoreButtonTimeout = null;
	}, restoreButtonDelay);
}
</script>

<div
	class="harper-editor @container flex h-full min-h-0 w-full flex-col overflow-hidden rounded-[10px] border-[0.5px] border-[rgba(28,26,22,0.14)] bg-[#fbfaf6] text-stone-950 shadow-2xl shadow-stone-950/5"
	style={editorStyle}
>
	<div class="flex min-h-0 min-w-0 flex-1 @max-[760px]:flex-col">
		<section class="relative min-w-0 flex-1 bg-[#fbfaf6]" aria-label="Document editor">
			<div class="h-full overflow-auto p-[34px_40px_56px] @max-[760px]:p-[28px_24px_42px]">
				<div class="mx-auto flex min-h-full max-w-[640px]">
					<div bind:this={editor} class="flex min-h-full w-full flex-1" spellcheck="false"></div>
				</div>
			</div>

			{#if !sidebarVisible && restoreButtonVisible}
				<button
					type="button"
					class="absolute top-3 right-3 z-20 inline-flex h-8 w-8 items-center justify-center rounded-md border-0 bg-transparent text-stone-600 shadow-none transition-colors duration-150 hover:text-stone-950 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-500"
					aria-label="Show problems sidebar"
					title="Show problems sidebar"
					in:fade={{ duration: 120 }}
					on:click={showSidebar}
				>
					<svg
						viewBox="0 0 20 20"
						aria-hidden="true"
						class="h-[18px] w-[18px] fill-none stroke-current stroke-[1.5] [stroke-linecap:round] [stroke-linejoin:round]"
					>
						<rect x="3.5" y="3" width="13" height="14" rx="3" />
						<path d="M12.5 3v14" />
					</svg>
				</button>
			{/if}
		</section>

		{#if sidebarVisible}
			<LintSidebar
				{lintBoxes}
				{activeLintId}
				focusLint={jumpTo}
				onActivate={(lintBox) => (activeLintId = lintBox == null ? null : lintBox.lint.context_hash)}
				onApplied={handleProblemAction}
				onIgnored={handleProblemAction}
				onIgnoreAll={ignoreAllProblems}
				onHideSidebar={hideSidebar}
			/>
		{/if}
	</div>

	<StatusBar
		text={documentText}
		problemCount={lintBoxes.length}
		{fontFamily}
		{fontSize}
		onFontFamilyChange={(next) => (fontFamily = next)}
		onFontSizeChange={(next) => (fontSize = normalizeFontSize(next))}
	/>
</div>

<script lang="ts">
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import Isolate from '$lib/components/Isolate.svelte';
import { zipSync, strToU8 } from 'fflate';

type FileEntry = {
	id: string;
	name: string;
	content: string;
};

type Toast = {
	id: number;
	title: string;
	body?: string;
	tone: 'success' | 'error' | 'info';
};

type WeirpackTestFailure = {
	expected: string;
	got: string;
};

type WeirpackTestFailures = Record<string, WeirpackTestFailure[]>;

const defaultManifest = {
	name: 'Weirpack Playground',
	author: 'Anonymous',
	version: '0.1.0',
	description: 'Exploring Weir rules in the browser.',
	license: 'MIT',
};

const defaultRule = `expr main (w/o)

let message "Use \`without\` instead of \`w/o\`"
let description "Expands the abbreviation \`w/o\` to the full word \`without\`."
let kind "Style"
let becomes "without"

test "She lacks w/o experience." "She lacks without experience."
test "He has w/o skills w/o knowledge." "He has without skills without knowledge."
`;

let nextId = 1;
let drawerOpen = true;
let renamingId: string | null = null;
let renameValue = '';
let activeFileId = 'file-1';
let activeContent = '';
let toasts: Toast[] = [];
let runningTests = false;
let linterReady = false;
let linter: import('harper.js').WorkerLinter | null = null;
let AceEditorComponent: typeof import('svelte-ace').AceEditor | null = null;
let editorReady = false;

let files: FileEntry[] = [
	{
		id: 'file-1',
		name: 'manifest.json',
		content: JSON.stringify(defaultManifest, null, 2),
	},
	{
		id: 'file-2',
		name: 'ExampleRule.weir',
		content: defaultRule,
	},
];

const editorOptions = {
	enableBasicAutocompletion: true,
	enableLiveAutocompletion: true,
	enableSnippets: true,
	showPrintMargin: false,
	wrap: true,
	fontFamily: '"JetBrains Mono", monospace',
	fontSize: '14px',
};

const modeByExtension: Record<string, string> = {
	json: 'json',
	js: 'javascript',
	ts: 'typescript',
	md: 'markdown',
	markdown: 'markdown',
	yaml: 'yaml',
	yml: 'yaml',
};

const makeId = () => {
	nextId += 1;
	return `file-${nextId}`;
};

$: {
	const file = files.find((entry) => entry.id === activeFileId);
	if (file && file.content !== activeContent) {
		activeContent = file.content;
	}
}

const getEditorMode = (name: string) => {
	const ext = name.split('.').pop()?.toLowerCase();
	if (!ext) {
		return 'text';
	}
	return modeByExtension[ext] ?? 'text';
};

const getActiveFile = () => files.find((entry) => entry.id === activeFileId) ?? null;

const setActiveFile = (id: string) => {
	activeFileId = id;
	renamingId = null;
};

const updateActiveContent = (value: string) => {
	activeContent = value;
	files = files.map((entry) =>
		entry.id === activeFileId ? { ...entry, content: value } : entry,
	);
};

const createFile = () => {
	const baseName = 'NewRule.weir';
	let candidate = baseName;
	let counter = 1;
	const names = new Set(files.map((entry) => entry.name));
	while (names.has(candidate)) {
		counter += 1;
		candidate = `NewRule${counter}.weir`;
	}
	const newFile = {
		id: makeId(),
		name: candidate,
		content: 'expr main',
	};
	files = [...files, newFile];
	setActiveFile(newFile.id);
};

const startRename = (file: FileEntry) => {
	renamingId = file.id;
	renameValue = file.name;
};

const commitRename = (file: FileEntry) => {
	const trimmed = renameValue.trim();
	if (!trimmed) {
		renamingId = null;
		return;
	}
	let candidate = trimmed;
	let counter = 1;
	const names = new Set(files.map((entry) => entry.name));
	names.delete(file.name);
	while (names.has(candidate)) {
		counter += 1;
		candidate = `${trimmed.replace(/\.[^/.]+$/, '')}-${counter}${trimmed.includes('.') ? '.' + trimmed.split('.').pop() : ''}`;
	}
	files = files.map((entry) =>
		entry.id === file.id ? { ...entry, name: candidate } : entry,
	);
	renamingId = null;
};

const deleteFile = (file: FileEntry) => {
	files = files.filter((entry) => entry.id !== file.id);
	if (activeFileId === file.id) {
		activeFileId = files[0]?.id ?? '';
	}
};

const pushToast = (toast: Omit<Toast, 'id'>) => {
	const id = Date.now() + Math.floor(Math.random() * 1000);
	toasts = [...toasts, { ...toast, id }];
	setTimeout(() => {
		toasts = toasts.filter((item) => item.id !== id);
	}, 4500);
};

const parseManifest = () => {
	const manifestFile = files.find((entry) => entry.name === 'manifest.json');
	if (!manifestFile) {
		return defaultManifest;
	}
	try {
		const parsed = JSON.parse(manifestFile.content);
		return parsed;
	} catch (error) {
		pushToast({
			title: 'manifest.json is invalid',
			body: 'Fix the JSON before running tests or downloading.',
			tone: 'error',
		});
		return null;
	}
};

const validateManifest = (manifest: Record<string, unknown>) => {
	const required = ['author', 'version', 'description', 'license'];
	for (const key of required) {
		if (typeof manifest[key] !== 'string' || manifest[key] === '') {
			pushToast({
				title: `Manifest field missing: ${key}`,
				body: 'Each field must be a non-empty string.',
				tone: 'error',
			});
			return false;
		}
	}
	return true;
};

const buildWeirpackBytes = () => {
	const manifest = parseManifest();
	if (!manifest || !validateManifest(manifest)) {
		return null;
	}
	const entries: Record<string, Uint8Array> = {
		'manifest.json': strToU8(JSON.stringify(manifest, null, 2)),
	};
	for (const entry of files) {
		if (entry.name === 'manifest.json') {
			continue;
		}
		entries[entry.name] = strToU8(entry.content);
	}
	return zipSync(entries, { level: 6 });
};

const runTests = async () => {
	if (!linter) {
		pushToast({
			title: 'Linter still loading',
			body: 'Give it a moment and try again.',
			tone: 'info',
		});
		return;
	}
	const bytes = buildWeirpackBytes();
	if (!bytes) {
		return;
	}
	runningTests = true;
	try {
		const failures = (await linter.loadWeirpackFromBytes(bytes)) as WeirpackTestFailures | undefined;
		if (!failures || Object.keys(failures).length === 0) {
			pushToast({
				title: 'All tests passed',
				body: 'Your Weirpack is green.',
				tone: 'success',
			});
		} else {
			for (const [ruleName, failuresForRule] of Object.entries(failures)) {
				for (const failure of failuresForRule) {
					pushToast({
						title: `${ruleName} failed`,
						body: `Expected "${failure.expected}" but got "${failure.got}".`,
						tone: 'error',
					});
				}
			}
		}
	} catch (error) {
		pushToast({
			title: 'Unable to run tests',
			body: 'The Weirpack could not be evaluated.',
			tone: 'error',
		});
	} finally {
		runningTests = false;
	}
};

const downloadWeirpack = () => {
	const bytes = buildWeirpackBytes();
	if (!bytes) {
		return;
	}
	const manifest = parseManifest() ?? defaultManifest;
	const baseName = String(manifest.name ?? 'weirpack').trim() || 'weirpack';
	const safeName = baseName.replace(/[^a-zA-Z0-9_-]/g, '-');
	const blob = new Blob([bytes], { type: 'application/zip' });
	const url = URL.createObjectURL(blob);
	const link = document.createElement('a');
	link.href = url;
	link.download = `${safeName}.weirpack`;
	link.click();
	URL.revokeObjectURL(url);
};

onMount(async () => {
	if (!browser) {
		return;
	}
	const [{ WorkerLinter, binary }, { AceEditor }] = await Promise.all([
		import('harper.js'),
		import('svelte-ace'),
	]);

	await Promise.all([
		import('brace/mode/json'),
		import('brace/mode/javascript'),
		import('brace/mode/markdown'),
		import('brace/mode/text'),
		import('brace/mode/typescript'),
		import('brace/mode/yaml'),
		import('brace/theme/chrome'),
	]);

	const newLinter = new WorkerLinter({ binary });
	await newLinter.setup();
	linter = newLinter;
	linterReady = true;
	AceEditorComponent = AceEditor;
	editorReady = true;
});
</script>

<Isolate>
	<div class="relative flex h-screen w-screen overflow-hidden bg-[#fef4e7] text-black">
		<div class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,_rgba(244,168,62,0.25),_transparent_55%)]"></div>

		<aside
			class={`relative z-10 flex h-full flex-col border-r border-black/10 bg-white/80 backdrop-blur transition-all duration-300 ${drawerOpen ? 'w-72' : 'w-14'}`}
		>
			<div class="flex items-center justify-between px-3 py-3">
				{#if drawerOpen}
					<div class="text-sm font-semibold uppercase tracking-wider text-black/70">Weirpack</div>
					<button
						class="flex h-8 w-8 items-center justify-center rounded-lg border border-black/10 bg-white text-black/70 hover:bg-black/5"
						on:click={() => (drawerOpen = false)}
						title="Collapse drawer"
						aria-label="Collapse drawer"
					>
						<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
							<path d="M11.78 4.22a.75.75 0 0 1 0 1.06L8.06 9l3.72 3.72a.75.75 0 1 1-1.06 1.06L6.47 9.53a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0z" />
						</svg>
					</button>
				{:else}
					<button
						class="mx-auto flex h-8 w-8 items-center justify-center rounded-lg border border-black/10 bg-white text-black/70 hover:bg-black/5"
						on:click={() => (drawerOpen = true)}
						title="Expand drawer"
						aria-label="Expand drawer"
					>
						<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
							<path d="M8.22 4.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 1 1-1.06-1.06L11.94 9 8.22 5.28a.75.75 0 0 1 0-1.06z" />
						</svg>
					</button>
				{/if}
			</div>

			{#if drawerOpen}
				<div class="px-3 pb-2">
					<button
						class="flex w-full items-center justify-center gap-2 rounded-lg border border-black/10 bg-black px-3 py-2 text-sm font-semibold uppercase tracking-wide text-white hover:bg-black/90"
						on:click={createFile}
					>
						<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
							<path d="M10 4a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 10 4z" />
						</svg>
						New file
					</button>
				</div>

				<div class="flex-1 overflow-auto px-2 pb-4">
					{#each files as file (file.id)}
						<div
							class={`group flex items-center justify-between rounded-lg px-2 py-2 text-sm ${file.id === activeFileId ? 'bg-white shadow-sm' : 'hover:bg-white/60'}`}
						>
							<button
								class="flex flex-1 items-center gap-2 text-left"
								on:click={() => setActiveFile(file.id)}
							>
								<span class="h-2 w-2 rounded-full bg-black/30"></span>
								{#if renamingId === file.id}
									<input
										class="w-full rounded-md border border-black/10 bg-white px-2 py-1 text-xs focus:border-black/40 focus:outline-none"
										bind:value={renameValue}
										on:keydown={(event) => {
											if (event.key === 'Enter') {
												commitRename(file);
											}
											if (event.key === 'Escape') {
												renamingId = null;
											}
										}}
										on:blur={() => commitRename(file)}
									/>
								{:else}
									<span class="truncate">{file.name}</span>
								{/if}
							</button>

							<div class="flex items-center gap-1 text-black/50 opacity-0 transition group-hover:opacity-100">
								<button
									class="rounded-md p-1 hover:bg-black/10"
									on:click={() => startRename(file)}
									title="Rename file"
									aria-label="Rename file"
								>
									<svg viewBox="0 0 20 20" class="h-3.5 w-3.5" fill="currentColor" aria-hidden="true">
										<path d="M13.64 2.86a1.5 1.5 0 0 1 2.12 2.12l-8.5 8.5-3.36.84.84-3.36 8.5-8.5z" />
										<path d="M11.5 4.99 15 8.5" stroke="currentColor" stroke-width="1.2" />
									</svg>
								</button>
								{#if files.length > 1}
									<button
										class="rounded-md p-1 hover:bg-black/10"
										on:click={() => deleteFile(file)}
										title="Delete file"
										aria-label="Delete file"
									>
										<svg viewBox="0 0 20 20" class="h-3.5 w-3.5" fill="currentColor" aria-hidden="true">
											<path d="M7.5 3a1 1 0 0 0-1 1v1H4.75a.75.75 0 0 0 0 1.5h.57l.6 9.01A2 2 0 0 0 7.91 17h4.18a2 2 0 0 0 1.99-1.49l.6-9.01h.57a.75.75 0 0 0 0-1.5H13.5V4a1 1 0 0 0-1-1h-5zM8 6h4v8H8V6z" />
										</svg>
									</button>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="flex flex-1 flex-col items-center gap-4 pt-6 text-xs text-black/50">
					<div class="rotate-90 text-xs font-semibold tracking-widest uppercase">Files</div>
				</div>
			{/if}
		</aside>

		<main class="relative z-10 flex flex-1 flex-col">
			<div class="flex items-center justify-between border-b border-black/10 bg-white/70 px-4 py-3">
				<div class="flex items-center gap-3">
					<div class="text-xs font-semibold uppercase tracking-[0.2em] text-black/50">Playground</div>
					<div class="text-sm font-medium text-black/80">
						{getActiveFile()?.name ?? 'No file selected'}
					</div>
				</div>
				<div class="text-xs uppercase tracking-[0.18em] text-black/40">
					{linterReady ? 'Weir runner online' : 'Warming up harper.js'}
				</div>
			</div>

			<div class="flex-1 overflow-hidden p-4">
				<div class="h-full rounded-2xl border border-black/10 bg-white shadow-[0_20px_60px_-40px_rgba(0,0,0,0.4)]">
					{#if editorReady && AceEditorComponent}
						<svelte:component
							this={AceEditorComponent}
							width="100%"
							height="100%"
							value={activeContent}
							lang={getEditorMode(getActiveFile()?.name ?? '')}
							theme="chrome"
							options={editorOptions}
							on:input={(event) => updateActiveContent(event.detail)}
						/>
					{:else}
						<div class="flex h-full items-center justify-center text-sm uppercase tracking-[0.3em] text-black/40">
							Loading editor…
						</div>
					{/if}
				</div>
			</div>
		</main>

		<div class="fixed bottom-6 right-6 z-20 flex items-center gap-3">
			<button
				class={`flex items-center gap-2 rounded-full border border-black/10 px-4 py-3 text-sm font-semibold uppercase tracking-wide shadow-lg transition ${runningTests ? 'bg-black text-white opacity-70' : 'bg-black text-white hover:bg-black/90'}`}
				on:click={runTests}
				disabled={runningTests}
			>
				<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
					<path d="M6.75 4.25a.75.75 0 0 1 .78.02l7.5 4.75a.75.75 0 0 1 0 1.26l-7.5 4.75A.75.75 0 0 1 6 14.5v-9a.75.75 0 0 1 .75-.75z" />
				</svg>
				{runningTests ? 'Running' : 'Run tests'}
			</button>
			<button
				class="flex items-center gap-2 rounded-full border border-black/10 bg-white px-4 py-3 text-sm font-semibold uppercase tracking-wide text-black shadow-lg hover:bg-black/5"
				on:click={downloadWeirpack}
			>
				<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
					<path d="M10 3.5a.75.75 0 0 1 .75.75v6.19l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 1 1 1.06-1.06l2.22 2.22V4.25A.75.75 0 0 1 10 3.5z" />
					<path d="M4 13.75a.75.75 0 0 1 .75-.75h10.5a.75.75 0 0 1 .75.75v1.5A2.75 2.75 0 0 1 13.25 18h-6.5A2.75 2.75 0 0 1 4 15.25v-1.5z" />
				</svg>
				Download
			</button>
		</div>

		<div class="fixed bottom-24 right-6 z-20 flex w-[320px] flex-col gap-3">
			{#each toasts as toast (toast.id)}
				<div
					class={`rounded-2xl border px-4 py-3 text-sm shadow-xl ${
						toast.tone === 'success'
							? 'border-green-200 bg-green-50 text-green-900'
							: toast.tone === 'error'
								? 'border-red-200 bg-red-50 text-red-900'
								: 'border-black/10 bg-white text-black'
					}`}
				>
					<div class="text-sm font-semibold">{toast.title}</div>
					{#if toast.body}
						<div class="mt-1 text-xs leading-snug text-black/70">{toast.body}</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
</Isolate>

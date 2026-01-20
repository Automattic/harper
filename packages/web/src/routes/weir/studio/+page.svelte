<script lang="ts">
import { packWeirpackFiles, unpackWeirpackBytes } from 'harper.js';
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import Isolate from '$lib/components/Isolate.svelte';
import Toasts, { type Toast } from '$lib/components/Toasts.svelte';
import WeirStudioStart from '$lib/components/WeirStudioStart.svelte';
import WeirStudioWorkspace from '$lib/components/WeirStudioWorkspace.svelte';

type WeirpackTestFailure = {
	expected: string;
	got: string;
};

type WeirpackTestFailures = Record<string, WeirpackTestFailure[]>;

const defaultManifest = {
	name: 'Weirpack Studio',
	author: 'Anonymous',
	version: '0.1.0',
	description: 'Exploring Weir rules in the browser.',
	license: 'MIT',
};

const newRuleTemplate = `expr main`

const defaultRule = `expr main (w/o)

let message "Use \`without\` instead of \`w/o\`"
let description "Expands the abbreviation \`w/o\` to the full word \`without\`."
let kind "Style"
let becomes "without"

test "She lacks w/o experience." "She lacks without experience."
test "He has w/o skills w/o knowledge." "He has without skills without knowledge."
`;

/** Used for generating new filenames */
let nextId = 2;

/** Is the drawer open? */
let drawerOpen = true;

/** The filename that is currently being renamed. */
let renamingId: string | null = null;

/** The current prospective value to rename the file to.
 Not relevant if renamingId is null */
let renameValue = '';

/** The name of the file currently in the viewport. */
let activeFileId : string | null = '';

let toasts: Toast[] = [];
let runningTests = false;
let linterReady = false;
let linter: import('harper.js').WorkerLinter | null = null;
let AceEditorComponent: typeof import('svelte-ace').AceEditor | null = null;
let editorReady = false;
let activeFile: FileEntry | null = null;
let packLoaded = false;
let fileInputEl: HTMLInputElement | null = null;
let saveTimeout: number | null = null;
let checkingStorage = true;

const storageKey = 'harper-weirpack-studio';

let files: Map<string, string> = new Map();

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

function createFileName(): string {
  return `NewRule-${nextId++}`;
}

function getEditorMode(name: string): string {
  let ext = name.split('.', 2)[1];
  if (ext == null){
    'text'
  }

  let mode = modeByExtension[ext];
  return mode;
}

function setActiveFile(id: string) {
  activeFile = id;
}

function updateActiveContent(value: string) {
  files.set(activeFile, value);
}

function createFile() {
  files.set(createFileName(), newRuleTemplate);
}

function deleteFile(file: string) {
  files.delete(file);
}

function pushToast(toast: Omit<Toast, 'id'>) {
	const id = Date.now() + Math.floor(Math.random() * 1000);
	toasts = [...toasts, { ...toast, id }];
	setTimeout(() => {
		toasts = toasts.filter((item) => item.id !== id);
	}, 4500);
}

function initializePack(entries: Map<string, string>) {
	files = entries;
	activeFileId = entries.keys().next().value ?? null;
	packLoaded = true;
}

function openExamplePack() {
	initializePack(new Map([
  [
			 'manifest.json',
			 JSON.stringify(defaultManifest, null, 2),
       ],
       [
			 'ExampleRule.weir',
			 defaultRule,
       ]
	]));
}

function createEmptyPack() {
	const manifest = {
		...defaultManifest,
		name: 'Untitled Weirpack',
	};
	initializePack(new Map([
  [
			 'manifest.json',
			 JSON.stringify(manifest, null, 2),
		],
	]));
}

function resetToStartScreen() {
	files = new Map();
	activeFileId = '';
	packLoaded = false;
	renamingId = null;
	if (browser) {
		localStorage.removeItem(storageKey);
	}
}

function loadWeirpackFromBytes(bytes: Uint8Array) {
	try {
		const unpacked = unpackWeirpackBytes(bytes);
		const entries: Map<string, string> = new Map([
			[
				 'manifest.json',
				 JSON.stringify(unpacked.manifest, null, 2),
			],
			...unpacked.rules.map((rule) => ([
				 rule.name,
				 rule.content,
			])),
		]);

		nextId = entries.size + 1;
		initializePack(entries);
	} catch (error) {
		pushToast({
			title: 'Unable to load Weirpack',
			body: 'Make sure the file is a valid .weirpack archive.',
			tone: 'error',
		});
	}
}

async function handleUpload(event: Event) {
	const input = event.currentTarget as HTMLInputElement;
	if (!input.files?.length) {
		return;
	}
	const file = input.files[0];
	const bytes = new Uint8Array(await file.arrayBuffer());
	loadWeirpackFromBytes(bytes);
	input.value = '';
}

function parseManifest() {
	if (!files.has('manifest.json')) {
		return defaultManifest;
	}
	try {
		const parsed = JSON.parse(files.get("manifest.json")!);
		return parsed;
	} catch (error) {
		pushToast({
			title: 'manifest.json is invalid',
			body: 'Fix the JSON before running tests or downloading.',
			tone: 'error',
		});
		return null;
	}
}

function validateManifest(manifest: Record<string, unknown>) {
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
}

function buildWeirpackBytes(): Uint8Array<ArrayBufferLike>|null {
	const manifest = parseManifest();
	if (!manifest || !validateManifest(manifest)) {
		return null;
	}

	const normalizedFiles = files.entries().map(([key, val]) =>
		key === 'manifest.json'
			? { ...entry, content: JSON.stringify(manifest, null, 2) }
			: entry,
	);

	try {
		return packWeirpackFiles(normalizedFiles);
	} catch (error) {
		pushToast({
			title: 'Weirpack export failed',
			body: 'manifest.json is required to save a Weirpack.',
			tone: 'error',
		});
		return null;
	}
}

function bytesToBase64(bytes: Uint8Array) {
	let binary = '';
	for (const byte of bytes) {
		binary += String.fromCharCode(byte);
	}
	return btoa(binary);
}

function base64ToBytes(base64: string) {
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i += 1) {
		bytes[i] = binary.charCodeAt(i);
	}
	return bytes;
}

function saveWeirpackToStorage() {
	if (!browser || !packLoaded) {
		return;
	}
	const bytes = buildWeirpackBytes();
	if (!bytes) {
		return;
	}
	try {
		localStorage.setItem(storageKey, bytesToBase64(bytes));
	} catch (error) {
		console.warn('Unable to store Weirpack', error);
	}
}

async function runTests() {
	if (!packLoaded) {
		pushToast({
			title: 'No Weirpack loaded',
			body: 'Choose a Weirpack to run tests.',
			tone: 'info',
		});
		return;
	}
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
				body: 'The tests in your Weirpack all pass.',
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
}

function downloadWeirpack() {
	if (!packLoaded) {
		pushToast({
			title: 'No Weirpack loaded',
			body: 'Choose a Weirpack to download.',
			tone: 'info',
		});
		return;
	}
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
}

onMount(async () => {
	if (!browser) {
		return;
	}
	const stored = localStorage.getItem(storageKey);
	if (stored) {
		try {
			const bytes = base64ToBytes(stored);
			loadWeirpackFromBytes(bytes);
		} catch (error) {
			console.warn('Unable to restore Weirpack', error);
		}
	}
	checkingStorage = false;

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

		<WeirStudioWorkspace
			{drawerOpen}
			{files}
			{activeFile}
			{editorReady}
			{AceEditorComponent}
			{editorOptions}
			{linterReady}
			{runningTests}
			{packLoaded}
			{renamingId}
			{renameValue}
			{getEditorMode}
			onToggleDrawer={() => (drawerOpen = !drawerOpen)}
			onCreateFile={createFile}
			onSelectFile={setActiveFile}
			onRenameFile={startRename}
			onDeleteFile={deleteFile}
			onUpdateContent={updateActiveContent}
			onRunTests={runTests}
			onDownload={downloadWeirpack}
			onClosePack={resetToStartScreen}
			onRenameValueChange={(value) => (renameValue = value)}
			onCommitRename={commitRename}
			onCancelRename={() => (renamingId = null)}
		/>

		<Toasts {toasts} />

		{#if !packLoaded}
			<WeirStudioStart
				onUpload={() => fileInputEl?.click()}
				onOpenExample={openExamplePack}
				onCreateEmpty={createEmptyPack}
				onUploadChange={handleUpload}
				bind:fileInputEl
				loading={checkingStorage}
				loadingLabel="Checking local storage for a saved Weirpack."
			/>
		{/if}
	</div>
</Isolate>

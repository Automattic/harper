<script lang="ts">
import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import Isolate from '$lib/components/Isolate.svelte';
import Toasts, { type Toast } from '$lib/components/Toasts.svelte';
import WeirStudioStart from '$lib/components/WeirStudioStart.svelte';
import WeirStudioWorkspace, {
	type FileEntry,
} from '$lib/components/WeirStudioWorkspace.svelte';

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

const defaultRule = `expr main (w/o)

let message "Use \`without\` instead of \`w/o\`"
let description "Expands the abbreviation \`w/o\` to the full word \`without\`."
let kind "Style"
let becomes "without"

test "She lacks w/o experience." "She lacks without experience."
test "He has w/o skills w/o knowledge." "He has without skills without knowledge."
`;

let nextId = 2;
let drawerOpen = true;
let renamingId: string | null = null;
let renameValue = '';
let activeFileId = '';
let activeContent = '';
let toasts: Toast[] = [];
let runningTests = false;
let linterReady = false;
let linter: import('harper.js').WorkerLinter | null = null;
let AceEditorComponent: typeof import('svelte-ace').AceEditor | null = null;
let editorReady = false;
let activeFile: FileEntry | null = null;
let packLoaded = false;
let fileInputEl: HTMLInputElement | null = null;

let files: FileEntry[] = [];

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
	let candidate = nextId;
	while (files.some((entry) => entry.id === `file-${candidate}`)) {
		candidate += 1;
	}
	nextId = candidate + 1;
	return `file-${candidate}`;
};

$: activeFile = files.find((entry) => entry.id === activeFileId) ?? null;
$: if (activeFile && activeFile.content !== activeContent) {
	activeContent = activeFile.content;
}

const getEditorMode = (name: string) => {
	const ext = name.split('.').pop()?.toLowerCase();
	if (!ext) {
		return 'text';
	}
	return modeByExtension[ext] ?? 'text';
};

const setActiveFile = (id: string) => {
	activeFileId = id;
	renamingId = null;
};

const updateActiveContent = (value: string) => {
	activeContent = value;
	files = files.map((entry) => (entry.id === activeFileId ? { ...entry, content: value } : entry));
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
	files = files.map((entry) => (entry.id === file.id ? { ...entry, name: candidate } : entry));
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

const initializePack = (entries: FileEntry[]) => {
	files = entries;
	activeFileId = entries[0]?.id ?? '';
	activeContent = entries[0]?.content ?? '';
	packLoaded = true;
};

const openExamplePack = () => {
	initializePack([
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
	]);
};

const createEmptyPack = () => {
	const manifest = {
		...defaultManifest,
		name: 'Untitled Weirpack',
	};
	initializePack([
		{
			id: 'file-1',
			name: 'manifest.json',
			content: JSON.stringify(manifest, null, 2),
		},
	]);
};

const resetToStartScreen = () => {
	files = [];
	activeFileId = '';
	activeContent = '';
	packLoaded = false;
	renamingId = null;
};

const loadWeirpackFromBytes = (bytes: Uint8Array) => {
	try {
		const archive = unzipSync(bytes);
		const manifestBytes = archive['manifest.json'];
		if (!manifestBytes) {
			pushToast({
				title: 'manifest.json missing',
				body: 'The weirpack must include a manifest.json file.',
				tone: 'error',
			});
			return;
		}

		const manifestText = strFromU8(manifestBytes);
		const manifest = JSON.parse(manifestText);
		const entries: FileEntry[] = [
			{
				id: 'file-1',
				name: 'manifest.json',
				content: JSON.stringify(manifest, null, 2),
			},
		];

		let counter = 1;
		for (const [name, data] of Object.entries(archive)) {
			if (name === 'manifest.json' || !name.endsWith('.weir')) {
				continue;
			}
			counter += 1;
			entries.push({
				id: `file-${counter}`,
				name,
				content: strFromU8(data as Uint8Array),
			});
		}

		nextId = entries.length + 1;
		initializePack(entries);
	} catch (error) {
		pushToast({
			title: 'Unable to load Weirpack',
			body: 'Make sure the file is a valid .weirpack archive.',
			tone: 'error',
		});
	}
};

const handleUpload = async (event: Event) => {
	const input = event.currentTarget as HTMLInputElement;
	if (!input.files?.length) {
		return;
	}
	const file = input.files[0];
	const bytes = new Uint8Array(await file.arrayBuffer());
	loadWeirpackFromBytes(bytes);
	input.value = '';
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
		const failures = (await linter.loadWeirpackFromBytes(bytes)) as
			| WeirpackTestFailures
			| undefined;
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
};

const downloadWeirpack = () => {
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

		<WeirStudioWorkspace
			{drawerOpen}
			{files}
			{activeFile}
			{activeContent}
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
			/>
		{/if}
	</div>
</Isolate>

<script lang="ts">
import { Button, Card } from 'components';
import type { AceEditor } from 'svelte-ace';
import CloseIcon from '$lib/components/icons/CloseIcon.svelte';
import DownloadIcon from '$lib/components/icons/DownloadIcon.svelte';
import PlayIcon from '$lib/components/icons/PlayIcon.svelte';
import WeirStudioFileExplorer, {} from '$lib/components/WeirStudioFileExplorer.svelte';

export let drawerOpen = true;
export let files: Map<string, string> = new Map();
export let activeFile: string | null = null;
export let editorReady = false;
export let AceEditorComponent: typeof AceEditor | null = null;
export let editorOptions: Record<string, unknown>;
export let linterReady = false;
export let runningTests = false;
export let packLoaded = false;

export let onToggleDrawer: () => void;
export let onCreateFile: () => void;
export let onSelectFile: (id: string) => void;
export let onRenameFile: (from: string, to: string) => void;
export let onDeleteFile: (file: string) => void;
export let onUpdateContent: (value: string) => void;
export let onRunTests: () => void;
export let onDownload: () => void;
export let onClosePack: () => void;

export let getEditorMode: (name: string) => string;
</script>

<WeirStudioFileExplorer
	{drawerOpen}
	{files}
	{activeFile}
	{onToggleDrawer}
	{onCreateFile}
	{onSelectFile}
	{onDeleteFile}
  {onRenameFile}
/>

<main class="relative z-10 flex flex-1 flex-col">
	<div class="flex items-center justify-between border-b border-black/10 bg-white/70 px-4 py-3">
		<div class="flex items-center gap-3">
			<div class="text-xs font-semibold uppercase tracking-[0.2em] text-black/50">Studio</div>
			<div class="text-sm font-medium text-black/80">{activeFile ?? 'No file selected'}</div>
		</div>
		<div class="flex items-center gap-3">
			<div class="text-xs uppercase tracking-[0.18em] text-black/40">
				{linterReady ? 'Weir runner online' : 'Warming up harper.js'}
			</div>
			<Button
				size="xs"
				color="white"
				className="h-8 w-8 !p-0"
				title="Close Weirpack"
				aria-label="Close Weirpack"
				on:click={onClosePack}
			>
				<CloseIcon className="h-3.5 w-3.5" />
			</Button>
		</div>
	</div>

	<div class="flex-1 overflow-hidden p-4">
		<Card className="h-full border-black/10 bg-white/95 p-0 shadow-[0_20px_60px_-40px_rgba(0,0,0,0.4)]">
			{#if editorReady && AceEditorComponent}
				{#key activeFile}
					<svelte:component
						this={AceEditorComponent}
						width="100%"
						height="100%"
						value={activeFile ? files.get(activeFile) : ""}
						lang={getEditorMode(activeFile ?? '')}
						theme="chrome"
						options={editorOptions}
						on:input={(event) => onUpdateContent(event.detail)}
					/>
				{/key}
			{:else}
				<div class="flex h-full items-center justify-center text-sm uppercase tracking-[0.3em] text-black/40">
					Loading editor…
				</div>
			{/if}
		</Card>
	</div>
</main>

<div class="fixed bottom-6 right-6 z-20 flex items-center gap-3">
	<Button
		size="md"
		color="dark"
		pill
		className={runningTests ? 'opacity-70' : undefined}
		on:click={onRunTests}
		disabled={!packLoaded || runningTests}
	>
		<PlayIcon className="h-4 w-4" />
		{runningTests ? 'Running' : 'Run tests'}
	</Button>
	<Button size="md" color="white" pill on:click={onDownload} disabled={!packLoaded}>
		<DownloadIcon className="h-4 w-4" />
		Download
	</Button>
</div>

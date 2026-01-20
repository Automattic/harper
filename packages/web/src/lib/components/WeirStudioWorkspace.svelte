<script lang="ts">
import { Button, Card, Input } from 'components';
import type { AceEditor } from 'svelte-ace';
import ChevronLeftIcon from '$lib/components/icons/ChevronLeftIcon.svelte';
import ChevronRightIcon from '$lib/components/icons/ChevronRightIcon.svelte';
import CloseIcon from '$lib/components/icons/CloseIcon.svelte';
import DownloadIcon from '$lib/components/icons/DownloadIcon.svelte';
import EditIcon from '$lib/components/icons/EditIcon.svelte';
import PlayIcon from '$lib/components/icons/PlayIcon.svelte';
import PlusIcon from '$lib/components/icons/PlusIcon.svelte';
import TrashIcon from '$lib/components/icons/TrashIcon.svelte';

export type FileEntry = {
	id: string;
	name: string;
	content: string;
};

export let drawerOpen = true;
export let files: FileEntry[] = [];
export let activeFile: FileEntry | null = null;
export let activeContent = '';
export let editorReady = false;
export let AceEditorComponent: typeof AceEditor | null = null;
export let editorOptions: Record<string, unknown>;
export let linterReady = false;
export let runningTests = false;
export let packLoaded = false;

export let onToggleDrawer: () => void;
export let onCreateFile: () => void;
export let onSelectFile: (id: string) => void;
export let onRenameFile: (file: FileEntry) => void;
export let onDeleteFile: (file: FileEntry) => void;
export let onUpdateContent: (value: string) => void;
export let onRunTests: () => void;
export let onDownload: () => void;
export let onClosePack: () => void;

export let renamingId: string | null = null;
export let renameValue = '';
export let onRenameValueChange: (value: string) => void;
export let onCommitRename: (file: FileEntry) => void;
export let onCancelRename: () => void;

export let getEditorMode: (name: string) => string;
</script>

<aside
	class={`relative z-10 flex h-full flex-col border-r border-black/10 bg-white/80 backdrop-blur transition-all duration-300 ${drawerOpen ? 'w-72' : 'w-14'}`}
>
	<div class="flex items-center justify-between px-3 py-3">
		{#if drawerOpen}
			<div class="text-sm font-semibold uppercase tracking-wider text-black/70">Weirpack</div>
			<Button
				size="xs"
				color="white"
				className="h-8 w-8 !p-0"
				on:click={onToggleDrawer}
				title="Collapse drawer"
				aria-label="Collapse drawer"
			>
				<ChevronLeftIcon className="h-4 w-4" />
			</Button>
		{:else}
			<Button
				size="xs"
				color="white"
				className="mx-auto h-8 w-8 !p-0"
				on:click={onToggleDrawer}
				title="Expand drawer"
				aria-label="Expand drawer"
			>
				<ChevronRightIcon className="h-4 w-4" />
			</Button>
		{/if}
	</div>

	{#if drawerOpen}
		<div class="px-3 pb-2">
			<Button
				color="dark"
				size="sm"
				className="w-full uppercase tracking-wide"
				on:click={onCreateFile}
				disabled={!packLoaded}
			>
				<PlusIcon className="h-4 w-4" />
				New file
			</Button>
		</div>

		<div class="flex-1 overflow-auto px-2 pb-4">
			{#each files as file (file.id)}
				<div
					class={`group flex items-center justify-between rounded-lg px-2 py-2 text-sm ${file.id === activeFile?.id ? 'bg-white shadow-sm' : 'hover:bg-white/60'}`}
				>
					<button
						class="flex flex-1 items-center gap-2 text-left"
						on:click={() => onSelectFile(file.id)}
					>
						<span class="h-2 w-2 rounded-full bg-black/30"></span>
						{#if renamingId === file.id}
							<Input
								size="sm"
								className="w-full text-xs"
								bind:value={renameValue}
								on:input={(event) => onRenameValueChange((event.target as HTMLInputElement).value)}
								on:keydown={(event) => {
									if (event.key === 'Enter') {
										onCommitRename(file);
									}
									if (event.key === 'Escape') {
										onCancelRename();
									}
								}}
								on:blur={() => onCommitRename(file)}
							/>
						{:else}
							<span class="truncate">{file.name}</span>
						{/if}
					</button>

					<div class="flex items-center gap-1 text-black/50 opacity-0 transition group-hover:opacity-100">
						<Button
							size="xs"
							color="white"
							className="h-6 w-6 !p-0"
							on:click={() => onRenameFile(file)}
							title="Rename file"
							aria-label="Rename file"
						>
							<EditIcon className="h-3.5 w-3.5" />
						</Button>
						{#if files.length > 1}
							<Button
								size="xs"
								color="white"
								className="h-6 w-6 !p-0"
								on:click={() => onDeleteFile(file)}
								title="Delete file"
								aria-label="Delete file"
							>
								<TrashIcon className="h-3.5 w-3.5" />
							</Button>
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
			<div class="text-xs font-semibold uppercase tracking-[0.2em] text-black/50">Studio</div>
			<div class="text-sm font-medium text-black/80">{activeFile?.name ?? 'No file selected'}</div>
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
				{#key activeFile?.id}
					<svelte:component
						this={AceEditorComponent}
						width="100%"
						height="100%"
						value={activeContent}
						lang={getEditorMode(activeFile?.name ?? '')}
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

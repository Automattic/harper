<script lang="ts">
import { Button, Card, Input } from 'components';
import type { AceEditor } from 'svelte-ace';

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
				<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
					<path d="M11.78 4.22a.75.75 0 0 1 0 1.06L8.06 9l3.72 3.72a.75.75 0 1 1-1.06 1.06L6.47 9.53a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0z" />
				</svg>
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
				<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
					<path d="M8.22 4.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 1 1-1.06-1.06L11.94 9 8.22 5.28a.75.75 0 0 1 0-1.06z" />
				</svg>
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
				<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
					<path d="M10 4a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 10 4z" />
				</svg>
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
							<svg viewBox="0 0 20 20" class="h-3.5 w-3.5" fill="currentColor" aria-hidden="true">
								<path d="M13.64 2.86a1.5 1.5 0 0 1 2.12 2.12l-8.5 8.5-3.36.84.84-3.36 8.5-8.5z" />
								<path d="M11.5 4.99 15 8.5" stroke="currentColor" stroke-width="1.2" />
							</svg>
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
								<svg viewBox="0 0 20 20" class="h-3.5 w-3.5" fill="currentColor" aria-hidden="true">
									<path d="M7.5 3a1 1 0 0 0-1 1v1H4.75a.75.75 0 0 0 0 1.5h.57l.6 9.01A2 2 0 0 0 7.91 17h4.18a2 2 0 0 0 1.99-1.49l.6-9.01h.57a.75.75 0 0 0 0-1.5H13.5V4a1 1 0 0 0-1-1h-5zM8 6h4v8H8V6z" />
								</svg>
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
				<svg viewBox="0 0 20 20" class="h-3.5 w-3.5" fill="currentColor" aria-hidden="true">
					<path d="M5.72 5.72a.75.75 0 0 1 1.06 0L10 8.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L11.06 10l3.22 3.22a.75.75 0 1 1-1.06 1.06L10 11.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L8.94 10 5.72 6.78a.75.75 0 0 1 0-1.06z" />
				</svg>
			</Button>
		</div>
	</div>

	<div class="flex-1 overflow-hidden p-4">
		<Card className="h-full border-black/10 bg-white/95 p-0 shadow-[0_20px_60px_-40px_rgba(0,0,0,0.4)]">
			{#if editorReady && AceEditorComponent}
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
		<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
			<path d="M6.75 4.25a.75.75 0 0 1 .78.02l7.5 4.75a.75.75 0 0 1 0 1.26l-7.5 4.75A.75.75 0 0 1 6 14.5v-9a.75.75 0 0 1 .75-.75z" />
		</svg>
		{runningTests ? 'Running' : 'Run tests'}
	</Button>
	<Button size="md" color="white" pill on:click={onDownload} disabled={!packLoaded}>
		<svg viewBox="0 0 20 20" class="h-4 w-4" fill="currentColor" aria-hidden="true">
			<path d="M10 3.5a.75.75 0 0 1 .75.75v6.19l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 1 1 1.06-1.06l2.22 2.22V4.25A.75.75 0 0 1 10 3.5z" />
			<path d="M4 13.75a.75.75 0 0 1 .75-.75h10.5a.75.75 0 0 1 .75.75v1.5A2.75 2.75 0 0 1 13.25 18h-6.5A2.75 2.75 0 0 1 4 15.25v-1.5z" />
		</svg>
		Download
	</Button>
</div>

<script lang="ts">
import { Button, Input } from 'components';
import ChevronLeftIcon from '$lib/components/icons/ChevronLeftIcon.svelte';
import ChevronRightIcon from '$lib/components/icons/ChevronRightIcon.svelte';
import EditIcon from '$lib/components/icons/EditIcon.svelte';
import PlusIcon from '$lib/components/icons/PlusIcon.svelte';
import TrashIcon from '$lib/components/icons/TrashIcon.svelte';

/** Whether to render the file explorer as a closed drawer or an open one. */
export let drawerOpen = true;
export let files: Map<string, string> = new Map();
/** The filename of the currently selected file. */
export let activeFile: string | null = null;

let renamingFile: string | null = null;
let renameValue = '';

export let onToggleDrawer: () => void;
export let onCreateFile: () => void;
export let onSelectFile: (id: string) => void;
export let onDeleteFile: (file: string) => void;
export let onRenameFile: (from: string, to: string) => void;
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
			>
				<PlusIcon className="h-4 w-4" />
				New file
			</Button>
		</div>

		<div class="flex-1 overflow-auto px-2 pb-4">
			{#each files as [file]}
				<div
					class={`group flex items-center justify-between rounded-lg px-2 py-2 text-sm ${file === activeFile? 'bg-white shadow-sm' : 'hover:bg-white/60'}`}
				>
					<div class="flex flex-1 items-center gap-2 text-left">
						<span class="h-2 w-2 rounded-full bg-black/30"></span>
						{#if renamingFile === file}
							<div class="flex-1">
								<Input
									size="sm"
									className="w-full text-xs"
									autofocus
									bind:value={renameValue}
									on:keydown={(event: KeyboardEvent) => {
										if (event.key === 'Enter') {
											onRenameFile(file, renameValue);
										}
										if (event.key === 'Escape') {
                      renamingFile = null;
										}
									}}
									on:blur={() => onRenameFile(file, renameValue)}
								/>
							</div>
						{:else}
							<button class="flex-1 truncate text-left" on:click={() => onSelectFile(file)}>
								{file}
							</button>
						{/if}
					</div>

					<div class="flex items-center gap-1 text-black/50 opacity-0 transition group-hover:opacity-100">
						<Button
							size="xs"
							color="white"
							className="h-6 w-6 !p-0"
							on:click={() => renameValue = file}
							title="Rename file"
							aria-label="Rename file"
						>
							<EditIcon className="h-3.5 w-3.5" />
						</Button>
						{#if files.size > 1}
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
			<div class="rotate-135 text-xs font-semibold tracking-widest uppercase">Files</div>
		</div>
	{/if}
</aside>

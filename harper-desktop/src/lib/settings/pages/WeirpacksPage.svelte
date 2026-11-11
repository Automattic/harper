<script lang="ts">
import {
	Button,
	IconButton,
	Input,
	PackageIcon,
	Panel,
	Toggle,
	TrashIcon,
	UploadIcon,
} from 'components';
import { onMount } from 'svelte';
import { Client, type InstalledWeirpack } from '../../client';

let weirpacks: InstalledWeirpack[] = [];
let busy = true;
let error = '';
let status = '';
let dragging = false;
let editingPackId: string | null = null;
let editingPackName = '';

$: enabledPacks = weirpacks.filter((pack) => pack.enabled);
$: enabledPackRules = enabledPacks.reduce((total, pack) => total + pack.ruleCount, 0);

onMount(() => {
	void run(async () => {});
});

async function run(action: () => Promise<void>, message = '') {
	busy = true;
	error = '';
	status = '';
	try {
		await action();
		status = message;
	} catch (cause) {
		error = String(cause);
	} finally {
		try {
			weirpacks = (await Client.getWeirpacks()).toSorted((a, b) => b.addedAt - a.addedAt);
		} catch (cause) {
			error = [error, String(cause)].filter(Boolean).join('\n');
		}
		busy = false;
	}
}

/** Install each selected archive independently and report partial failures without hiding successes. */
async function installFiles(files: File[]) {
	if (busy || files.length === 0) return;
	await run(async () => {
		const failures: string[] = [];
		for (const file of files) {
			try {
				if (!file.name.toLowerCase().endsWith('.weirpack'))
					throw new Error('Select a .weirpack file.');
				await Client.installWeirpack(file.name, new Uint8Array(await file.arrayBuffer()));
			} catch (cause) {
				failures.push(`${file.name}: ${String(cause)}`);
			}
		}
		if (failures.length) throw new Error(failures.join('\n'));
	}, 'Packs installed. Enabled packs apply to the editor and highlighter.');
}

function startRenamePack(pack: InstalledWeirpack) {
	editingPackId = pack.id;
	editingPackName = pack.name;
}

async function commitRenamePack() {
	const id = editingPackId;
	if (!id || busy) return;
	editingPackId = null;
	await run(() => Client.updateWeirpack(id, { name: editingPackName }), 'Pack renamed.');
}

function formatSize(bytes: number) {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}
</script>

<section aria-busy={busy}>
    <div class="stanza">
        <div class="eyebrow">Weirpacks</div>
        <p class="section-copy">Custom rules and dictionaries for the editor and system-wide highlighter.</p>
        <label
            class:dragging
            class="drop-zone"
            on:dragover={(event) => { event.preventDefault(); dragging = !busy; }}
            on:dragleave={() => (dragging = false)}
            on:drop={(event) => {
                event.preventDefault();
                dragging = false;
                void installFiles(Array.from(event.dataTransfer?.files ?? []));
            }}
        >
            <span class="big-mark purple"><UploadIcon className="control-icon" /></span>
            <span class="grow">
                <strong>{dragging ? 'Drop to install' : 'Install a Weirpack'}</strong>
                <p>Drag .weirpack files here, or click to browse. Multiple files are OK.</p>
            </span>
            <span class="button primary">Choose files...</span>
            <input type="file" accept=".weirpack" multiple disabled={busy} on:change={(event) => {
                const files = Array.from(event.currentTarget.files ?? []);
                event.currentTarget.value = '';
                void installFiles(files);
            }} />
        </label>
        {#if busy}<p role="status">Loading packs...</p>{/if}
        {#if error}<p role="alert" style="white-space: pre-line">{error}</p>{/if}
        {#if status}<p role="status">{status}</p>{/if}
        <p class="muted">{weirpacks.length} packs installed, {enabledPacks.length} active, {enabledPackRules} extra rules.</p>
    </div>
    <div class="stanza">
        <div class="eyebrow">Installed packs</div>
        <Panel>
            {#each weirpacks as pack (pack.id)}
                <div class:disabled={!pack.enabled} class="pack-row">
                    <div class="pack-icon"><PackageIcon className="settings-icon" /></div>
                    <div class="grow">
                        {#if editingPackId === pack.id}
                            <Input unstyled class="inline-edit" type="text" aria-label="Pack name" bind:value={editingPackName}
                                on:blur={commitRenamePack}
                                on:keydown={(event) => {
                                    if (event.detail.key === 'Enter') void commitRenamePack();
                                    if (event.detail.key === 'Escape') editingPackId = null;
                                }} />
                        {:else}
                            <Button unstyled class="pack-title" type="button" disabled={busy}
                                title="Rename pack" on:click={() => startRenamePack(pack)}>{pack.name}</Button>
                        {/if}
                        <p><code>{pack.filename}</code> - v{pack.version} - {pack.ruleCount} rules - {formatSize(pack.size)} - added {new Date(pack.addedAt).toLocaleDateString()}</p>
                    </div>
                    <IconButton danger disabled={busy} aria-label={`Remove ${pack.name}`}
                        on:click={() => run(() => Client.removeWeirpack(pack.id), 'Pack removed.')}>
                        <TrashIcon className="control-icon" />
                    </IconButton>
                    <Toggle appearance="settings" checked={pack.enabled} disabled={busy}
                        aria-label={`Toggle ${pack.name}`}
                        on:click={() => run(() => Client.updateWeirpack(pack.id, { enabled: !pack.enabled }), 'Pack updated.')} />
                </div>
            {:else}
                {#if !busy}<p class="muted">No custom Weirpacks installed.</p>{/if}
            {/each}
        </Panel>
    </div>
</section>

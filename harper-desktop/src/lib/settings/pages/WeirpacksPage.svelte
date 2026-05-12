<script lang="ts">
import { createInitialSettingsState, type Weirpack } from '../settings-data';

let weirpacks = createInitialSettingsState().weirpacks;

let packDragState: 'idle' | 'dragging' = 'idle';
let editingPackId: string | null = null;
let editingPackName = '';

$: enabledPackCount = weirpacks.filter((pack) => pack.enabled).length;
$: enabledPackRules = weirpacks
	.filter((pack) => pack.enabled)
	.reduce((total, pack) => total + pack.ruleCount, 0);

function updatePack(id: string, patch: Partial<Weirpack>) {
	weirpacks = weirpacks.map((pack) => (pack.id === id ? { ...pack, ...patch } : pack));
}

function removePack(id: string) {
	weirpacks = weirpacks.filter((pack) => pack.id !== id);
}

function formatSize(bytes: number) {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function formatDate(iso: string) {
	return new Date(iso).toLocaleDateString(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
	});
}

function startRenamePack(pack: Weirpack) {
	editingPackId = pack.id;
	editingPackName = pack.name;
}

function commitRenamePack() {
	if (!editingPackId) return;
	const name = editingPackName.trim();

	if (name) {
		updatePack(editingPackId, { name });
	}

	editingPackId = null;
	editingPackName = '';
}
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">Weirpacks</div>
          <p class="section-copy">
            Bundles of custom rules that can be layered on top of Harper's built-in checks.
          </p>
          <p class="result-summary">Weirpack installation is not wired yet.</p>

          <label
            class:dragging={packDragState === "dragging"}
            class="drop-zone"
            on:dragover={(event) => {
              event.preventDefault();
              packDragState = "dragging";
            }}
            on:dragleave={() => (packDragState = "idle")}
            on:drop={(event) => {
              event.preventDefault();
              packDragState = "idle";
            }}
          >
            <span class="big-mark purple">
              <span class="settings-icon icon-upload" aria-hidden="true"></span>
            </span>
            <span class="grow">
              <strong>{packDragState === "dragging" ? "Drop to install" : "Install a Weirpack"}</strong>
              <p>Drag a .weirpack file here, or click to browse. Multiple files are OK.</p>
            </span>
            <span class="button primary">Choose file...</span>
            <input
              type="file"
              accept=".weirpack,.json,.wpck"
              multiple
              disabled
            />
          </label>

          <p class="muted">
            {weirpacks.length} packs installed, {enabledPackCount} active, {enabledPackRules}
            extra rules.
          </p>
        </div>

        <div class="stanza">
          <div class="eyebrow">Installed packs</div>
          <div class="list-card">
            {#each weirpacks as pack}
              <div class:disabled={!pack.enabled} class="pack-row">
                <div class="pack-icon">
                  <span class="settings-icon icon-package" aria-hidden="true"></span>
                </div>
                <div class="grow">
                  {#if editingPackId === pack.id}
                    <input
                      class="inline-edit"
                      type="text"
                      bind:value={editingPackName}
                      on:blur={commitRenamePack}
                      on:keydown={(event) => {
                        if (event.key === "Enter") commitRenamePack();
                        if (event.key === "Escape") {
                          editingPackId = null;
                          editingPackName = "";
                        }
                      }}
                    />
                  {:else}
                    <button class="pack-title" type="button" disabled title="Not wired yet" on:dblclick={() => startRenamePack(pack)}>
                      {pack.name}
                    </button>
                  {/if}
                  <p>
                    <code>{pack.filename}</code>
                    <span> - {pack.ruleCount} rules - {formatSize(pack.size)} - added {formatDate(pack.addedAt)}</span>
                  </p>
                  {#if pack.installState !== "installed"}
                    <span class={`status ${pack.installState}`}>{pack.installState}</span>
                  {/if}
                </div>
                <button
                  class="icon-button danger"
                  type="button"
                  disabled
                  title="Not wired yet"
                  aria-label={`Remove ${pack.name}`}
                  on:click={() => removePack(pack.id)}
                >
                  <span class="settings-icon icon-trash" aria-hidden="true"></span>
                </button>
                <button
                  class:checked={pack.enabled}
                  class="toggle"
                  type="button"
                  role="switch"
                  disabled
                  title="Not wired yet"
                  aria-checked={pack.enabled}
                  aria-label={`Toggle ${pack.name}`}
                  on:click={() => updatePack(pack.id, { enabled: !pack.enabled })}
                >
                  <span></span>
                </button>
              </div>
            {/each}
          </div>
        </div>
      </section>

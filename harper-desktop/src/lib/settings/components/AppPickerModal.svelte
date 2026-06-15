<script lang="ts">
import { invoke } from '@tauri-apps/api/core';

export let bundleId = '';
export let existingBundleIds: string[];
export let isSaving = false;
export let close: () => void;
export let add: (bundleId: string) => void;

let searchResults: Array<{ name: string; bundle_id: string }> = [];
let isSearching = false;
let debounceTimeout: number | null = null;

$: trimmedBundleId = bundleId.trim();
$: isDuplicate = existingBundleIds.includes(trimmedBundleId);
$: canAdd = Boolean(trimmedBundleId) && !isDuplicate && !isSaving;

async function performSearch(query: string) {
	if (!query.trim()) {
		searchResults = [];
		return;
	}

	isSearching = true;
	try {
		const results = await invoke<Array<{ name: string; bundle_id: string }>>('search_apps', {
			query,
		});
		searchResults = results;
	} catch (error) {
		console.error('Search failed:', error);
		searchResults = [];
	} finally {
		isSearching = false;
	}
}

function handleInput(event: Event) {
	const target = event.target as HTMLInputElement;
	bundleId = target.value;

	if (debounceTimeout) {
		clearTimeout(debounceTimeout);
	}

	debounceTimeout = window.setTimeout(() => {
		performSearch(bundleId);
	}, 300);
}

function selectApp(selectedBundleId: string) {
	bundleId = selectedBundleId;
	searchResults = [];
}

function submit() {
	if (canAdd) {
		add(trimmedBundleId);
	}
}
</script>

<div
  class="modal-backdrop"
  role="button"
  tabindex="0"
  aria-label="Close application picker"
  on:click={close}
  on:keydown={(event) => {
    if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
      close();
    }
  }}
>
  <div
    class="modal"
    role="dialog"
    tabindex="-1"
    aria-label="Choose an application"
    on:click|stopPropagation={() => {}}
    on:keydown|stopPropagation={(event) => {
      if (event.key === "Escape") {
        close();
      }
    }}
  >
    <div class="modal-head">
      <strong>Add application</strong>
      <span>Enter the app bundle ID Harper should watch.</span>
    </div>
    <div class="modal-search">
      <span class="settings-icon icon-search" aria-hidden="true"></span>
      <input
        type="text"
        placeholder="Search for an app..."
        value={bundleId}
        disabled={isSaving}
        on:input={handleInput}
        on:keydown={(event) => {
          if (event.key === "Enter") {
            submit();
          }
        }}
      />
    </div>
    <div class="modal-list">
      {#if isSearching}
        <div class="empty">Searching...</div>
      {:else if searchResults.length > 0}
        {#each searchResults as result}
          <div
            class="app-result"
            role="button"
            tabindex="0"
            on:click={() => selectApp(result.bundle_id)}
            on:keydown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                selectApp(result.bundle_id);
              }
            }}
          >
            <div class="app-result-name">{result.name}</div>
            <div class="app-result-bundle-id">{result.bundle_id}</div>
          </div>
        {/each}
      {:else if trimmedBundleId}
        {#if isDuplicate}
          <div class="empty">That application is already configured.</div>
        {:else}
          <div class="empty">No matching apps found. Try typing the bundle ID directly (e.g., com.apple.TextEdit)</div>
        {/if}
      {:else}
        <div class="empty">Search for an app by name, or enter the bundle ID directly.</div>
      {/if}
    </div>
    <div class="modal-actions">
      <button class="button" type="button" on:click={close}>Cancel</button>
      <button class="button primary" type="button" disabled={!canAdd} on:click={submit}>Add</button>
    </div>
  </div>
</div>

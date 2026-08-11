<script lang="ts">
import { open } from '@tauri-apps/plugin-dialog';
import { onMount } from 'svelte';
import { type AppSearchResult, Client } from '$lib/client';
import AppIcon from './AppIcon.svelte';

export let bundleId = '';
export let existingBundleIds: string[];
export let isSaving = false;
export let close: () => void;
export let add: (bundleId: string) => void;

let searchResults: AppSearchResult[] = [];
let isSearching = false;
let isBrowsing = false;
let supportsBrowse = false;
let pickerError = '';
let debounceTimeout: number | null = null;
let searchRequestId = 0;
let browseRequestId = 0;

$: trimmedBundleId = bundleId.trim();
$: isDuplicate = existingBundleIds.includes(trimmedBundleId);
$: canAdd = Boolean(trimmedBundleId) && !isDuplicate && !isSaving && !isBrowsing;

onMount(() => {
	let mounted = true;

	void performSearch(bundleId);
	void Client.supportsAppBrowse()
		.then((supported) => {
			if (mounted) {
				supportsBrowse = supported;
			}
		})
		.catch((error) => console.error('Unable to check Browse support:', error));

	return () => {
		mounted = false;
		cancelPendingWork();
	};
});

function clearDebounce() {
	if (debounceTimeout !== null) {
		window.clearTimeout(debounceTimeout);
		debounceTimeout = null;
	}
}

function cancelPendingSearch() {
	clearDebounce();
	searchRequestId += 1;
	isSearching = false;
}

function cancelPendingWork() {
	cancelPendingSearch();
	browseRequestId += 1;
	isBrowsing = false;
}

async function performSearch(query: string) {
	const requestId = ++searchRequestId;

	isSearching = true;
	try {
		const results = await Client.searchApps(query);

		if (requestId === searchRequestId && query === bundleId) {
			searchResults = results;
		}
	} catch (error) {
		console.error('Search failed:', error);

		if (requestId === searchRequestId) {
			searchResults = [];
		}
	} finally {
		if (requestId === searchRequestId) {
			isSearching = false;
		}
	}
}

function handleInput(event: Event) {
	const target = event.target as HTMLInputElement;
	bundleId = target.value;
	pickerError = '';
	cancelPendingSearch();
	isSearching = true;

	const query = bundleId;
	debounceTimeout = window.setTimeout(() => {
		debounceTimeout = null;
		void performSearch(query);
	}, 300);
}

function selectApp(result: AppSearchResult) {
	cancelPendingWork();
	pickerError = '';
	bundleId = result.bundle_id;
	searchResults = [result];
}

async function browseForApp() {
	const requestId = ++browseRequestId;
	isBrowsing = true;

	try {
		const selectedPath = await open({
			title: 'Choose an application',
			defaultPath: '/Applications',
			multiple: false,
			directory: false,
			filters: [{ name: 'Applications', extensions: ['app'] }],
		});

		if (requestId !== browseRequestId || typeof selectedPath !== 'string') {
			return;
		}

		cancelPendingSearch();
		pickerError = '';
		const result = await Client.resolveAppPath(selectedPath);

		if (requestId === browseRequestId) {
			selectApp(result);
		}
	} catch (error) {
		console.error('Unable to use selected application:', error);

		if (requestId === browseRequestId) {
			pickerError = `Unable to use that application: ${error}`;
		}
	} finally {
		if (requestId === browseRequestId) {
			isBrowsing = false;
		}
	}
}

function handleClose() {
	cancelPendingWork();
	close();
}

function submit() {
	if (canAdd) {
		cancelPendingWork();
		add(trimmedBundleId);
	}
}
</script>

<div
  class="modal-backdrop"
  role="button"
  tabindex="0"
  aria-label="Close application picker"
  on:click={handleClose}
  on:keydown={(event) => {
    if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
      handleClose();
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
        handleClose();
      }
    }}
  >
    <div class="modal-head">
      <strong>Add application</strong>
      <span>Search by app name or bundle ID.{#if supportsBrowse} Browse if the app is not listed.{/if}</span>
    </div>
    <div class="modal-search">
      <span class="settings-icon icon-search" aria-hidden="true"></span>
      <input
        type="text"
        placeholder="App name or bundle ID"
        value={bundleId}
        disabled={isSaving || isBrowsing}
        on:input={handleInput}
        on:keydown={(event) => {
          if (event.key === "Enter") {
            submit();
          }
        }}
      />
      {#if supportsBrowse}
        <button
          class="button"
          type="button"
          disabled={isSaving || isBrowsing}
          on:click={browseForApp}
        >{isBrowsing ? "Browsing..." : "Browse..."}</button>
      {/if}
    </div>
    {#if pickerError}
      <div class="modal-error" role="alert">{pickerError}</div>
    {/if}
    <div class="modal-list">
      {#if isSearching}
        <div class="empty">Searching...</div>
      {:else if searchResults.length > 0}
        {#each searchResults as result}
          <div
            class="app-result"
            role="button"
            tabindex="0"
            on:click={() => selectApp(result)}
            on:keydown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                selectApp(result);
              }
            }}
          >
            <AppIcon bundleId={result.bundle_id} name={result.name} />
            <div class="app-result-copy">
              <div class="app-result-name">{result.name}</div>
              <div class="app-result-bundle-id">{result.bundle_id}</div>
            </div>
          </div>
        {/each}
      {:else if trimmedBundleId}
        {#if isDuplicate}
          <div class="empty">That application is already configured.</div>
        {:else if supportsBrowse}
          <div class="empty">No matching apps found. Search by app name or bundle ID, or use Browse.</div>
        {:else}
          <div class="empty">No matching apps found. Search by app name or bundle ID.</div>
        {/if}
      {:else if supportsBrowse}
        <div class="empty">Search by app name or bundle ID. If the app is not listed, use Browse.</div>
      {:else}
        <div class="empty">Search by app name or bundle ID.</div>
      {/if}
    </div>
    <div class="modal-actions">
      <button class="button" type="button" on:click={handleClose}>Cancel</button>
      <button class="button primary" type="button" disabled={!canAdd} on:click={submit}>Add</button>
    </div>
  </div>
</div>

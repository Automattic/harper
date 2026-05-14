<script lang="ts">
import { onMount } from 'svelte';
import { Client } from '$lib/client';

let debounceMs = 0;
let pendingDebounceMs = '0';
let isDebounceLoading = true;
let isDebounceSaving = false;
let debounceError = '';

onMount(() => {
	void loadDebounceMs();

	const refreshSettings = () => {
		if (!isDebounceSaving) {
			void loadDebounceMs();
		}
	};

	window.addEventListener('focus', refreshSettings);

	return () => {
		window.removeEventListener('focus', refreshSettings);
	};
});

async function loadDebounceMs() {
	isDebounceLoading = true;
	debounceError = '';

	try {
		debounceMs = await Client.getDebounceMs();
		pendingDebounceMs = String(debounceMs);
	} catch (error) {
		debounceError = `Unable to load debounce delay: ${error}`;
	} finally {
		isDebounceLoading = false;
	}
}

async function saveDebounceMs() {
	const parsed = Number(pendingDebounceMs);

	if (!Number.isInteger(parsed) || parsed < 0) {
		debounceError = 'Debounce delay must be a non-negative whole number.';
		pendingDebounceMs = String(debounceMs);
		return;
	}

	const previousDebounceMs = debounceMs;
	debounceMs = parsed;
	pendingDebounceMs = String(parsed);
	isDebounceSaving = true;
	debounceError = '';

	try {
		await Client.setDebounceMs(parsed);
	} catch (error) {
		debounceMs = previousDebounceMs;
		pendingDebounceMs = String(previousDebounceMs);
		debounceError = `Unable to save debounce delay: ${error}`;
	} finally {
		isDebounceSaving = false;
	}
}
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">Writing</div>
          <p class="section-copy">
            Choose how long Harper waits after text changes before checking it. Use 0 ms for
            immediate checking.
          </p>
          <div class="rows">
            <div class="inline-row">
              <label for="debounce-ms">Debounce delay:</label>
              <input
                id="debounce-ms"
                class="select"
                type="number"
                min="0"
                step="50"
                disabled={isDebounceLoading || isDebounceSaving}
                value={pendingDebounceMs}
                on:input={(event) => (pendingDebounceMs = event.currentTarget.value)}
                on:change={saveDebounceMs}
              />
              <span>ms</span>
            </div>
            {#if isDebounceLoading}
              <p class="result-summary">Loading debounce delay...</p>
            {:else if debounceError}
              <p class="result-summary">{debounceError}</p>
            {:else if isDebounceSaving}
              <p class="result-summary">Saving debounce delay...</p>
            {/if}
          </div>
        </div>
      </section>

<script lang="ts">
import type { Dialect } from 'harper.js';
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { DIALECT_OPTIONS } from '../settings-data';

const DialectValue = {
	American: 0,
	British: 1,
	Australian: 2,
	Canadian: 3,
	Indian: 4,
} as const;

let menuBar = true;
let menuBarClick = 'open-settings';
let launchAtStartup = false;
let autoUpdate = true;
let dialect = 'american';
let isDialectLoading = true;
let isDialectSaving = false;
let dialectError = '';
let isLaunchAtStartupLoading = true;
let isLaunchAtStartupSaving = false;
let launchAtStartupError = '';
let debounceMs = 0;
let debounceMsInput = '0';
let isDebounceLoading = true;
let isDebounceSaving = false;
let debounceError = '';

onMount(() => {
	void loadDialect();
	void loadLaunchAtStartup();
	void loadDebounceMs();

	const refreshSettings = () => {
		if (!isDialectSaving) {
			void loadDialect();
		}

		if (!isLaunchAtStartupSaving) {
			void loadLaunchAtStartup();
		}

		if (!isDebounceSaving) {
			void loadDebounceMs();
		}
	};

	window.addEventListener('focus', refreshSettings);

	return () => {
		window.removeEventListener('focus', refreshSettings);
	};
});

async function loadDialect() {
	isDialectLoading = true;
	dialectError = '';

	try {
		dialect = dialectToSettingsValue(await Client.getDialect());
	} catch (error) {
		dialectError = `Unable to load dialect: ${error}`;
	} finally {
		isDialectLoading = false;
	}
}

async function setDialect(value: string) {
	const previousDialect = dialect;

	dialect = value;
	isDialectSaving = true;
	dialectError = '';

	try {
		await Client.setDialect(settingsValueToDialect(value));
	} catch (error) {
		dialect = previousDialect;
		dialectError = `Unable to save dialect: ${error}`;
	} finally {
		isDialectSaving = false;
	}
}

async function loadLaunchAtStartup() {
	isLaunchAtStartupLoading = true;
	launchAtStartupError = '';

	try {
		launchAtStartup = await Client.getLaunchAtStartup();
	} catch (error) {
		launchAtStartupError = `Unable to load startup setting: ${error}`;
	} finally {
		isLaunchAtStartupLoading = false;
	}
}

async function setLaunchAtStartup(enabled: boolean) {
	const previousLaunchAtStartup = launchAtStartup;

	launchAtStartup = enabled;
	isLaunchAtStartupSaving = true;
	launchAtStartupError = '';

	try {
		await Client.setLaunchAtStartup(enabled);
	} catch (error) {
		launchAtStartup = previousLaunchAtStartup;
		launchAtStartupError = `Unable to save startup setting: ${error}`;
	} finally {
		isLaunchAtStartupSaving = false;
	}
}

async function loadDebounceMs() {
	isDebounceLoading = true;
	debounceError = '';

	try {
		debounceMs = await Client.getDebounceMs();
		debounceMsInput = String(debounceMs);
	} catch (error) {
		debounceError = `Unable to load debounce delay: ${error}`;
	} finally {
		isDebounceLoading = false;
	}
}

async function saveDebounceMs() {
	const parsedDebounceMs = Number(debounceMsInput);

	if (!Number.isInteger(parsedDebounceMs) || parsedDebounceMs < 0) {
		debounceError = 'Debounce delay must be a non-negative whole number.';
		debounceMsInput = String(debounceMs);
		return;
	}

	const previousDebounceMs = debounceMs;
	debounceMs = parsedDebounceMs;
	debounceMsInput = String(parsedDebounceMs);
	isDebounceSaving = true;
	debounceError = '';

	try {
		await Client.setDebounceMs(parsedDebounceMs);
	} catch (error) {
		debounceMs = previousDebounceMs;
		debounceMsInput = String(previousDebounceMs);
		debounceError = `Unable to save debounce delay: ${error}`;
	} finally {
		isDebounceSaving = false;
	}
}

function dialectToSettingsValue(dialect: Dialect): string {
	switch (dialect) {
		case DialectValue.British:
			return 'british';
		case DialectValue.Canadian:
			return 'canadian';
		case DialectValue.Australian:
			return 'australian';
		case DialectValue.Indian:
			return 'indian';
		default:
			return 'american';
	}
}

function settingsValueToDialect(value: string): Dialect {
	switch (value) {
		case 'british':
			return DialectValue.British as Dialect;
		case 'canadian':
			return DialectValue.Canadian as Dialect;
		case 'australian':
			return DialectValue.Australian as Dialect;
		case 'indian':
			return DialectValue.Indian as Dialect;
		default:
			return DialectValue.American as Dialect;
	}
}
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">General</div>
          <div class="rows">
            <div class="row top">
              <div>
                <strong>Keep Harper in the menu bar</strong>
                <p>Shows the Harper icon so you can open settings without opening the main app.</p>
              </div>
              <button
                class:checked={menuBar}
                class="checkbox"
                type="button"
                role="checkbox"
                disabled
                title="Not wired yet"
                aria-checked={menuBar}
              >
                {#if menuBar}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>

            <div class="inline-row">
              <label for="menu-bar-click">Click the icon to:</label>
              <select
                id="menu-bar-click"
                class="select"
                disabled
                title="Not wired yet"
                bind:value={menuBarClick}
              >
                <option value="open-settings">Open settings</option>
                <option value="show-menu">Show a menu</option>
                <option value="toggle-pause">Pause or resume</option>
                <option value="quick-review">Open quick review</option>
              </select>
            </div>

            <div class="row">
              <div>
                <strong>Launch Harper at startup</strong>
                <p>Harper will start silently when you log in.</p>
              </div>
              <button
                class:checked={launchAtStartup}
                class="checkbox"
                type="button"
                role="checkbox"
                disabled={isLaunchAtStartupLoading || isLaunchAtStartupSaving}
                aria-checked={launchAtStartup}
                on:click={() => setLaunchAtStartup(!launchAtStartup)}
              >
                {#if launchAtStartup}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>
            {#if isLaunchAtStartupLoading}
              <p class="result-summary">Loading startup setting...</p>
            {:else if launchAtStartupError}
              <p class="result-summary">{launchAtStartupError}</p>
            {:else if isLaunchAtStartupSaving}
              <p class="result-summary">Saving startup setting...</p>
            {/if}

            <div class="row top">
              <div>
                <strong>Automatically check for updates</strong>
                <p>Harper will check for new versions weekly.</p>
              </div>
              <button
                class:checked={autoUpdate}
                class="checkbox"
                type="button"
                role="checkbox"
                disabled
                title="Not wired yet"
                aria-checked={autoUpdate}
              >
                {#if autoUpdate}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Language</div>
          <p class="section-copy">
            Choose the dialect Harper uses to check spelling and grammar.
          </p>
          <div class="inline-row">
            <label for="dialect">English dialect:</label>
            <select
              id="dialect"
              class="select wide"
              disabled={isDialectLoading || isDialectSaving}
              bind:value={dialect}
              on:change={(event) => setDialect(event.currentTarget.value)}
            >
              {#each DIALECT_OPTIONS as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>
          {#if isDialectLoading}
            <p class="result-summary">Loading dialect...</p>
          {:else if dialectError}
            <p class="result-summary">{dialectError}</p>
          {:else if isDialectSaving}
            <p class="result-summary">Saving dialect...</p>
          {/if}
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Writing</div>
          <p class="section-copy">
            Choose how long Harper waits after text changes before checking it. Use 0 ms for
            immediate checking.
          </p>
          <div class="inline-row">
            <label for="debounce-ms">Debounce delay:</label>
            <input
              id="debounce-ms"
              class="select"
              type="number"
              min="0"
              step="50"
              disabled={isDebounceLoading || isDebounceSaving}
              value={debounceMsInput}
              on:input={(event) => (debounceMsInput = event.currentTarget.value)}
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

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Updates</div>
          <div class="row top">
            <div>
              <strong>You're up to date</strong>
              <p>Harper 1.4.2, released April 18, 2026.</p>
            </div>
            <button class="button" type="button" disabled title="Not wired yet">Check now</button>
          </div>
        </div>
      </section>

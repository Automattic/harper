<script lang="ts">
import { Dialect } from 'harper.js';
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { DIALECT_OPTIONS } from '../settings-data';

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

onMount(() => {
	void loadDialect();
	void loadLaunchAtStartup();

	const refreshSettings = () => {
		if (!isDialectSaving) {
			void loadDialect();
		}

		if (!isLaunchAtStartupSaving) {
			void loadLaunchAtStartup();
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

function dialectToSettingsValue(dialect: Dialect): string {
	switch (dialect) {
		case Dialect.British:
			return 'british';
		case Dialect.Canadian:
			return 'canadian';
		case Dialect.Australian:
			return 'australian';
		case Dialect.Indian:
			return 'indian';
		default:
			return 'american';
	}
}

function settingsValueToDialect(value: string): Dialect {
	switch (value) {
		case 'british':
			return Dialect.British;
		case 'canadian':
			return Dialect.Canadian;
		case 'australian':
			return Dialect.Australian;
		case 'indian':
			return Dialect.Indian;
		default:
			return Dialect.American;
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

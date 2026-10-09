<script lang="ts">
import { Button, Card, Input, Select, Textarea } from 'components';
import {
	Dialect,
	type LintConfig,
	type StructuredLintConfig,
	type StructuredLintSetting,
} from 'harper.js';
import logo from '/logo.png';
import ProtocolClient from '../ProtocolClient';
import type { Hotkey, Modifier, WeirpackMeta } from '../protocol';
import { ActivationKey } from '../protocol';
import { type ImportMode, type ImportSummary, summarizeImport } from '../settings/apply';
import { parseSettings } from '../settings/schema';
import StructuredRuleSettings from './StructuredRuleSettings.svelte';

let lintConfig: LintConfig = $state({});
let structuredLintConfig: StructuredLintConfig = $state({ settings: [] });
let lintDescriptions: Record<string, string> = $state({});
let searchQuery = $state('');
let searchQueryLower = $derived(searchQuery.toLowerCase());
let expandedGroups: Record<string, boolean> = $state({});
let dialect = $state(Dialect.American);
let isolateEnglish = $state(false);
let delay = $state(0);
let delayLoaded = $state(false);
let defaultEnabled = $state(false);
let activationKey: ActivationKey = $state(ActivationKey.Off);
let userDict = $state('');
let modifyHotkeyButton: Button;
let hotkey: Hotkey = $state({ modifiers: ['Ctrl'], key: 'e' });
let anyRulesEnabled = $derived(Object.values(lintConfig ?? {}).some((value) => value !== false));
let weirpacks: WeirpackMeta[] = $state([]);
let weirpackBusy = $state(false);
let weirpackError = $state('');

$effect(() => {
	ProtocolClient.setLintConfig($state.snapshot(lintConfig));
});

$effect(() => {
	ProtocolClient.setDialect(dialect);
});

$effect(() => {
	if (delayLoaded) {
		ProtocolClient.setDelay(delay);
	}
});

$effect(() => {
	ProtocolClient.setDefaultEnabled(defaultEnabled);
});

$effect(() => {
	ProtocolClient.setActivationKey(activationKey);
});

$effect(() => {
	ProtocolClient.setUserDictionary(stringToDict(userDict));
});

Promise.all([
	ProtocolClient.getLintConfig(),
	ProtocolClient.getStructuredLintConfig(),
	ProtocolClient.getLintDescriptions(),
]).then(([nextLintConfig, nextStructuredConfig, nextLintDescriptions]) => {
	lintConfig = nextLintConfig;
	structuredLintConfig = nextStructuredConfig;
	lintDescriptions = nextLintDescriptions;
});

ProtocolClient.getDialect().then((d) => {
	dialect = d;
});

ProtocolClient.getIsolateEnglish().then((value) => {
	isolateEnglish = value;
});

ProtocolClient.getDelay().then((value) => {
	delay = value;
	delayLoaded = true;
});

ProtocolClient.getDefaultEnabled().then((d) => {
	defaultEnabled = d;
});

ProtocolClient.getActivationKey().then((d) => {
	activationKey = d;
});

ProtocolClient.getHotkey().then((d) => {
	hotkey = {
		modifiers: [...d.modifiers],
		key: d.key,
	};
	buttonText = `Hotkey: ${d.modifiers.join('+')}+${d.key}`;
});

ProtocolClient.getUserDictionary().then((d) => {
	userDict = dictToString(d.toSorted());
});

ProtocolClient.getWeirpacks().then((stored) => {
	weirpacks = stored.toSorted((a, b) => b.installedAt.localeCompare(a.installedAt));
});

/** Converts the content of a text area to viable dictionary values. */
export function stringToDict(s: string): string[] {
	return s
		.split('\n')
		.map((s) => s.trim())
		.filter((v) => v.length > 0);
}

/** Converts the content of a text area to viable dictionary values. */
export function dictToString(values: string[]): string {
	return values.map((v) => v.trim()).join('\n');
}

function resetRulesToDefaults(): void {
	const keys = Object.keys(lintConfig ?? {});
	if (keys.length === 0) return;

	const nextConfig: LintConfig = { ...lintConfig };
	for (const key of keys) {
		nextConfig[key] = null;
	}
	lintConfig = nextConfig;
}

function updateAllRules(enabled: boolean): void {
	const keys = Object.keys(lintConfig ?? {});
	if (keys.length === 0) {
		return;
	}

	const nextConfig: LintConfig = { ...lintConfig };
	for (const key of keys) {
		nextConfig[key] = enabled;
	}
	lintConfig = nextConfig;
}

function toggleAllRules(): void {
	updateAllRules(!anyRulesEnabled);
}

function collectStructuredRuleNames(settings: StructuredLintSetting[]): string[] {
	const out: string[] = [];

	for (const setting of settings) {
		if ('Bool' in setting) {
			out.push(setting.Bool.name);
			continue;
		}

		if ('OneOfMany' in setting) {
			out.push(...setting.OneOfMany.names);
			continue;
		}

		out.push(...collectStructuredRuleNames(setting.Group.child.settings));
	}

	return out;
}

function buildDisplaySettings(
	structured: StructuredLintConfig,
	flat: LintConfig,
): StructuredLintSetting[] {
	const settings = [...(structured.settings ?? [])];
	const knownRules = new Set(collectStructuredRuleNames(settings));
	const extraRuleNames = Object.keys(flat)
		.filter((name) => !knownRules.has(name))
		.sort();

	if (extraRuleNames.length === 0) {
		return settings;
	}

	settings.push({
		Group: {
			label: 'Additional Rules',
			description: 'Rules present in the flat config but not yet assigned to a curated category.',
			child: {
				settings: extraRuleNames.map(
					(name) =>
						({
							Bool: {
								name,
								state: flat[name] ?? false,
								label: name,
							},
						}) as StructuredLintSetting,
				),
			},
		},
	});

	return settings;
}

let displayStructuredSettings: StructuredLintSetting[] = $state([]);

$effect(() => {
	displayStructuredSettings = buildDisplaySettings(structuredLintConfig, lintConfig);
});

function updateLintConfig(nextConfig: LintConfig) {
	lintConfig = nextConfig;
}

function setIsolateEnglishFromCheckbox(event: Event): void {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		console.warn('Could not update isolate English setting: missing checkbox input.');
		return;
	}

	isolateEnglish = input.checked;
	ProtocolClient.setIsolateEnglish(input.checked);
}

function toggleGroup(groupKey: string) {
	expandedGroups = {
		...expandedGroups,
		[groupKey]: !expandedGroups[groupKey],
	};
}

let importFileInput: HTMLInputElement;
let importMode: ImportMode = $state('merge');
let includeExtension = $state(true);
let importBusy = $state(false);
let importError = $state('');
let exportBusy = $state(false);
let exportError = $state('');
let pendingImportText: string | null = $state(null);
let pendingImportName = $state('');
let importPreview: ImportSummary | null = $state(null);
let showImportSuccess = $state(false);

try {
	if (sessionStorage.getItem('harper-settings-imported') === '1') {
		sessionStorage.removeItem('harper-settings-imported');
		showImportSuccess = true;
	}
} catch {
	// Storage may be unavailable; the success notice is best-effort.
}

async function handleExportSettings() {
	exportError = '';
	exportBusy = true;
	try {
		const settings = await ProtocolClient.exportSettings();
		const json = JSON.stringify(settings, null, 2);

		const blob = new Blob([json], { type: 'application/json;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `harper-settings-${new Date().toISOString().slice(0, 10)}.json`;
		document.body.appendChild(a);
		a.click();
		a.remove();
		URL.revokeObjectURL(url);
	} catch (e) {
		console.error('Failed to export settings:', e);
		exportError = 'Could not export settings. Please try again.';
	} finally {
		exportBusy = false;
	}
}

async function handleImportFileSelect(event: Event) {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		return;
	}

	const file = input.files?.[0];
	importError = '';
	importPreview = null;
	pendingImportText = null;
	pendingImportName = '';

	if (!file) {
		return;
	}

	importBusy = true;
	try {
		const text = await file.text();
		const incoming = parseSettings(text);
		const current = await ProtocolClient.exportSettings();
		if (current.extension === undefined) {
			throw new Error('Could not read the current settings.');
		}
		importPreview = summarizeImport(
			{ core: current.core, extension: current.extension },
			incoming,
			{ mode: importMode, includeExtension },
		);
		pendingImportText = text;
		pendingImportName = file.name;
	} catch (error) {
		importError = error instanceof Error ? error.message : 'Could not read that settings file.';
		input.value = '';
	} finally {
		importBusy = false;
	}
}

async function refreshImportPreview() {
	if (pendingImportText == null) {
		return;
	}

	try {
		const incoming = parseSettings(pendingImportText);
		const current = await ProtocolClient.exportSettings();
		if (current.extension === undefined) {
			return;
		}
		importPreview = summarizeImport(
			{ core: current.core, extension: current.extension },
			incoming,
			{ mode: importMode, includeExtension },
		);
	} catch {
		// Preview is best-effort; confirmation re-validates through the background worker.
	}
}

function setImportModeFromRadio(event: Event) {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		return;
	}

	importMode = input.value === 'replace' ? 'replace' : 'merge';
	void refreshImportPreview();
}

function setIncludeExtensionFromCheckbox(event: Event) {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		return;
	}

	includeExtension = input.checked;
	void refreshImportPreview();
}

async function confirmImport() {
	if (pendingImportText == null) {
		return;
	}

	importError = '';
	importBusy = true;
	try {
		await ProtocolClient.importSettings(pendingImportText, importMode, includeExtension);
		try {
			sessionStorage.setItem('harper-settings-imported', '1');
		} catch {
			// Best-effort; the reload below still applies the import.
		}
		window.location.reload();
	} catch (error) {
		importError = error instanceof Error ? error.message : 'Could not import settings.';
		importBusy = false;
	}
}

function cancelImport() {
	pendingImportText = null;
	pendingImportName = '';
	importPreview = null;
	importError = '';
	if (importFileInput) {
		importFileInput.value = '';
	}
}

let buttonText = $state('Set Hotkey');
let isBlue = $state(false); // modify color of hotkey button once it is pressed
function startHotkeyCapture(_modifyHotkeyButton: Button) {
	buttonText = 'Press desired hotkey combination now.';

	const handleKeydown = (event: KeyboardEvent) => {
		event.preventDefault();

		const modifiers: Modifier[] = [];
		if (event.ctrlKey) modifiers.push('Ctrl');
		if (event.shiftKey) modifiers.push('Shift');
		if (event.altKey) modifiers.push('Alt');

		let key = event.key;

		if (key !== 'Control' && key !== 'Shift' && key !== 'Alt') {
			if (modifiers.length === 0) {
				return;
			}
			buttonText = `Hotkey: ${modifiers.join('+')}+${key}`;
			// Create a plain object to avoid proxy cloning issues
			const newHotkey = {
				modifiers: [...modifiers],
				key: key,
			};

			hotkey = newHotkey;

			// Call ProtocolClient directly with the plain object to avoid proxy issues
			ProtocolClient.setHotkey(newHotkey);

			// Remove listener
			window.removeEventListener('keydown', handleKeydown);

			// change button color
			isBlue = !isBlue;
		}
	};

	// Add temporary key listener
	window.addEventListener('keydown', handleKeydown);
}

async function refreshWeirpacks() {
	const stored = await ProtocolClient.getWeirpacks();
	weirpacks = stored.toSorted((a, b) => b.installedAt.localeCompare(a.installedAt));
}

async function handleWeirpackUpload(event: Event) {
	const input = event.currentTarget as HTMLInputElement | null;
	const files = input?.files;
	if (!files || files.length === 0) {
		return;
	}

	weirpackError = '';
	weirpackBusy = true;
	try {
		for (const file of files) {
			const bytes = new Uint8Array(await file.arrayBuffer());
			await ProtocolClient.addWeirpack(file.name, bytes);
		}
		await refreshWeirpacks();
	} catch (error) {
		const message = error instanceof Error ? error.message : 'Failed to upload Weirpack.';
		weirpackError = message;
	} finally {
		weirpackBusy = false;
		input.value = '';
	}
}

async function removeWeirpack(id: string) {
	weirpackBusy = true;
	weirpackError = '';
	try {
		await ProtocolClient.removeWeirpack(id);
		await refreshWeirpacks();
	} catch (error) {
		const message = error instanceof Error ? error.message : 'Failed to remove Weirpack.';
		weirpackError = message;
	} finally {
		weirpackBusy = false;
	}
}
</script>

<!-- centered wrapper with side gutters -->
<div class="min-h-screen px-4 py-10">
  <div class="mx-auto max-w-screen-lg space-y-4">
    <Card class="flex items-center gap-3">
      <div class="flex h-9 w-9 items-center justify-center rounded-xl">
        <img src={logo} alt="Harper logo" class="h-5 w-auto" />
      </div>
      <div class="flex flex-col">
        <h1 class="text-base tracking-wide font-serif">Harper</h1>
        <p class="text-xs">Settings</p>
      </div>
    </Card>

    <!-- ── GENERAL ───────────────────────────── -->
    <Card class="space-y-6">
      <h2 class="pb-1 text-xs uppercase tracking-wider">General</h2>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <h3 class="text-sm">English Dialect</h3>
          <Select size="sm" class="w-44" bind:value={dialect}>
            <option value={Dialect.American}>🇺🇸 American</option>
            <option value={Dialect.British}>🇬🇧 British</option>
            <option value={Dialect.Australian}>🇦🇺 Australian</option>
            <option value={Dialect.Canadian}>🇨🇦 Canadian</option>
            <option value={Dialect.Indian}>🇮🇳 Indian</option>
          </Select>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">Ignore Non-English Text</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Skip text that Harper detects as not English.
            </p>
          </div>
          <input
            type="checkbox"
            checked={isolateEnglish}
            onchange={setIsolateEnglishFromCheckbox}
            class="h-5 w-5"
          />
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">Enable on New Sites by Default</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Can make some apps behave abnormally.
            </p>
          </div>
          <input
            type="checkbox"
            bind:checked={defaultEnabled}
            class="h-5 w-5"
          />
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between gap-4">
          <div class="flex flex-col">
            <h3 class="text-sm">Delay</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Wait this many milliseconds after typing stops before refreshing
              highlights.
            </p>
          </div>
          <input
            type="number"
            min="0"
            step="50"
            bind:value={delay}
            class="w-44 rounded-lg border border-cream-200 bg-white px-3 py-2.5 text-sm text-gray-900 shadow-sm outline-none transition focus:border-cream-300 focus:ring-2 focus:ring-primary-300 dark:border-cream-700 dark:bg-cream-900 dark:text-white dark:focus:border-cream-600 dark:focus:ring-primary-600"
          />
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">Activation Key</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              If you're finding that you're accidentally triggering Harper.
            </p>
          </div>
          <Select size="sm" class="w-44" bind:value={activationKey}>
            <option value={ActivationKey.Shift}>Double Shift</option>
            <option value={ActivationKey.Control}>Double Control</option>
            <option value={ActivationKey.Off}>Off</option>
          </Select>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">Apply Last Suggestion Hotkey</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Applies suggestion to last highlighted word.
            </p>
          </div>
          <Textarea readonly bind:value={buttonText} />
          <Button
            size="sm"
            color="light"
            style="background-color: {isBlue ? 'blue' : ''}"
            bind:this={modifyHotkeyButton}
            on:click={() => {
              startHotkeyCapture(modifyHotkeyButton);
              isBlue = !isBlue;
            }}>Modify Hotkey</Button
          >
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">User Dictionary</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Each word should be on its own line.
            </p>
          </div>
          <Textarea bind:value={userDict}></Textarea>
        </div>
      </div>
    </Card>

    <Card class="space-y-4">
      <h2 class="pb-1 text-xs uppercase tracking-wider">Backup & Restore</h2>

      {#if showImportSuccess}
        <p class="text-sm text-green-700 dark:text-green-400">
          Settings imported. This page reloaded to show the new values.
        </p>
      {/if}

      <div class="flex items-center justify-between gap-4">
        <div class="flex flex-col">
          <h3 class="text-sm">Export settings</h3>
          <p class="text-xs text-gray-600 dark:text-gray-400">
            Downloads a harper-settings.json file with your dialect, rules,
            dictionary, ignored suggestions, and extension settings.
          </p>
        </div>
        <Button size="sm" disabled={exportBusy} on:click={handleExportSettings}
          >{exportBusy ? 'Exporting…' : 'Export'}</Button
        >
      </div>

      {#if exportError}
        <p class="text-xs text-red-700 dark:text-red-400">{exportError}</p>
      {/if}

      <div class="flex items-center justify-between gap-4">
        <div class="flex flex-col">
          <h3 class="text-sm">Import settings</h3>
          <p class="text-xs text-gray-600 dark:text-gray-400">
            Restores settings from a harper-settings.json file. Nothing changes
            until you confirm below.
          </p>
        </div>
        <input
          type="file"
          accept=".json,application/json"
          bind:this={importFileInput}
          onchange={handleImportFileSelect}
          class="block w-1/4 text-sm file:rounded-md file:border-0 file:bg-primary file:text-white"
        />
      </div>

      {#if pendingImportText != null}
        <div class="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
          <label class="flex items-center gap-2">
            <input
              type="radio"
              name="import-mode"
              value="merge"
              checked={importMode === 'merge'}
              onchange={setImportModeFromRadio}
              class="h-4 w-4"
            />
            Merge
          </label>
          <label class="flex items-center gap-2">
            <input
              type="radio"
              name="import-mode"
              value="replace"
              checked={importMode === 'replace'}
              onchange={setImportModeFromRadio}
              class="h-4 w-4"
            />
            Replace
          </label>
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              checked={includeExtension}
              onchange={setIncludeExtensionFromCheckbox}
              class="h-4 w-4"
            />
            Include extension settings
          </label>
        </div>

        {#if importPreview != null}
          <ul class="list-disc space-y-1 pl-5 text-sm text-gray-600 dark:text-gray-400">
            <li>
              {importPreview.dialectChanged
                ? 'Dialect will change.'
                : 'Dialect stays the same.'}
              {pendingImportName ? ` File: ${pendingImportName}.` : ''}
            </li>
            <li>
              {importPreview.wordsAdded} word(s) added, {importPreview.wordsRemoved} removed.
            </li>
            <li>{importPreview.ruleChanges} rule setting(s) will change.</li>
            <li>{importPreview.ignoredLintsAdded} ignored suggestion(s) added.</li>
            {#if importPreview.extensionIncluded}
              <li>{importPreview.domainChanges} domain setting(s) will change.</li>
            {:else}
              <li>Extension settings will be left alone.</li>
            {/if}
          </ul>
          <div class="flex gap-3">
            <Button size="sm" disabled={importBusy} on:click={confirmImport}>
              {importBusy ? 'Importing…' : 'Confirm import'}
            </Button>
            <Button size="sm" color="light" disabled={importBusy} on:click={cancelImport}>
              Cancel
            </Button>
          </div>
        {/if}
      {/if}

      {#if importError}
        <p class="text-xs text-red-700 dark:text-red-400">{importError}</p>
      {/if}
    </Card>

    <Card class="space-y-4">
      <h2 class="pb-1 text-xs uppercase tracking-wider">Weirpacks</h2>

      <div class="space-y-2 flex flex-row w-full justify-between">
        <p class="text-xs text-gray-600 dark:text-gray-400">
          Upload one or more <code>.weirpack</code> files to add custom rule
          packs.
          <a href="https://writewithharper.com/docs/weir#Weirpacks"
            >What is a Weirpack?</a
          >
        </p>
        <input
          type="file"
          accept=".weirpack,application/zip"
          multiple
          disabled={weirpackBusy}
          onchange={handleWeirpackUpload}
          class="block w-1/4 text-sm file:rounded-md file:border-0 file:bg-primary file:text-white disabled:opacity-50"
        />
      </div>

      {#if weirpackError}
        <p class="text-xs text-red-700 dark:text-red-400">{weirpackError}</p>
      {/if}

      {#if weirpacks.length === 0}
        <p class="text-sm text-gray-600 dark:text-gray-400">
          No Weirpacks installed.
        </p>
      {:else}
        <div class="space-y-3">
          {#each weirpacks as weirpack}
            <div
              class="flex items-center justify-between gap-3 rounded-md border border-primary-100 p-3"
            >
              <div class="min-w-0">
                <p class="truncate text-sm">
                  {weirpack.name}{weirpack.version
                    ? ` v${weirpack.version}`
                    : ""}
                </p>
                <p class="truncate text-xs text-gray-600 dark:text-gray-400">
                  {weirpack.filename}
                </p>
              </div>
              <Button
                size="sm"
                color="light"
                disabled={weirpackBusy}
                on:click={() => removeWeirpack(weirpack.id)}
              >
                Remove
              </Button>
            </div>
          {/each}
        </div>
      {/if}
    </Card>

    <!-- ── RULES ─────────────────────────────── -->
    <Card class="space-y-4">
      <div class="flex items-center justify-between gap-4">
        <h2 class="text-xs uppercase tracking-wider">Rules</h2>
        <Input
          bind:value={searchQuery}
          placeholder="Search for a rule…"
          size="sm"
          class="w-60"
        />
      </div>
      <div class="flex flex-wrap gap-3">
        <Button size="sm" on:click={resetRulesToDefaults}
          >Reset to Default Rules</Button
        >
        <Button size="sm" on:click={toggleAllRules}>
          {anyRulesEnabled ? "Disable All Rules" : "Enable All Rules"}
        </Button>
      </div>

      <div class="rule-scroll space-y-4 max-h-80 overflow-y-auto pr-1">
        {#key displayStructuredSettings.length}
          <StructuredRuleSettings
            settings={displayStructuredSettings}
            {lintConfig}
            {lintDescriptions}
            {searchQueryLower}
            {expandedGroups}
            handleLintConfigChange={updateLintConfig}
            handleToggleGroup={toggleGroup}
          />
        {/key}
      </div>
    </Card>
  </div>
</div>

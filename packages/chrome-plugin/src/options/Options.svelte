<script lang="ts">
import { Button, Card, Input, Select, Textarea } from 'components';
import {
	type Dialect,
	type DialectInfo,
	type LintConfig,
	type StructuredLintConfig,
	type StructuredLintSetting,
} from 'harper.js';
import logo from '/logo.png';
import { CYCLE_LANGUAGE_COMMAND, DEFAULT_CYCLE_SHORTCUT } from '../commands';
import {
	codeFlag,
	dialectInfo,
	groupByLanguage,
	languageLabel,
	MAX_LANGUAGE_CYCLE,
} from '../languages';
import ProtocolClient from '../ProtocolClient';
import type { Hotkey, Modifier, WeirpackMeta } from '../protocol';
import { ActivationKey } from '../protocol';
import StructuredRuleSettings from './StructuredRuleSettings.svelte';

let lintConfig: LintConfig = $state({});
let structuredLintConfig: StructuredLintConfig = $state({ settings: [] });
let lintDescriptions: Record<string, string> = $state({});
let searchQuery = $state('');
let searchQueryLower = $derived(searchQuery.toLowerCase());
let expandedGroups: Record<string, boolean> = $state({});
let dialectCatalog: DialectInfo[] = $state([]);
let languageCycle: Dialect[] = $state([]);
let languageToAdd: Dialect | '' = $state('');
let availableLanguages = $derived(
	dialectCatalog.filter((info) => !languageCycle.includes(info.dialect)),
);
let cycleShortcut = $state('');
let cycleShortcutLoaded = $state(false);
/** `commands.update` exists in Firefox and Thunderbird; Chrome changes shortcuts on its own page. */
const commandsApi = chrome.commands as typeof chrome.commands & {
	update?: (detail: { name: string; shortcut: string }) => Promise<void>;
	openShortcutSettings?: () => Promise<void>;
};
let cycleShortcutEditable = typeof commandsApi?.update === 'function';
/**
 * Firefox 137+ opens its own shortcut page, which marks shortcuts used more than once. An
 * extension cannot see other extensions' shortcuts, so this is the only way to show a conflict.
 */
let canOpenShortcutSettings = typeof commandsApi?.openShortcutSettings === 'function';
let capturingCycleShortcut = $state(false);
let cycleShortcutError = $state('');
let isolateEnglish = $state(false);
let rememberLanguage = $state(true);
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

ProtocolClient.getDialectCatalog().then((catalog) => {
	dialectCatalog = catalog;
});

ProtocolClient.getLanguageCycle().then((cycle) => {
	languageCycle = cycle;
});

refreshCycleShortcut();

chrome.storage.local.get({ rememberLanguage: true }).then((value) => {
	rememberLanguage = value.rememberLanguage !== false;
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

function setRememberLanguageFromCheckbox(event: Event): void {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		return;
	}

	rememberLanguage = input.checked;
	chrome.storage.local.set({ rememberLanguage });
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

async function exportEnabledDomainsCSV() {
	try {
		const enabledDomains = await ProtocolClient.getEnabledDomains();
		const json = JSON.stringify(enabledDomains, null, 2);

		const blob = new Blob([json], { type: 'application/json;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'enabled-domains.json';
		document.body.appendChild(a);
		a.click();
		a.remove();
		URL.revokeObjectURL(url);
	} catch (e) {
		console.error('Failed to export enabled domains JSON:', e);
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

function saveLanguageCycle(next: Dialect[]): void {
	languageCycle = next;
	ProtocolClient.setLanguageCycle(next);
}

function addLanguage(): void {
	if (languageToAdd === '' || languageCycle.length >= MAX_LANGUAGE_CYCLE) {
		return;
	}

	saveLanguageCycle([...languageCycle, languageToAdd]);
	languageToAdd = '';
}

function moveLanguage(index: number, by: number): void {
	const target = index + by;
	if (target < 0 || target >= languageCycle.length) {
		return;
	}

	const next = [...languageCycle];
	[next[index], next[target]] = [next[target], next[index]];
	saveLanguageCycle(next);
}

function removeLanguage(index: number): void {
	if (languageCycle.length <= 1) {
		return;
	}

	saveLanguageCycle(languageCycle.filter((_, i) => i !== index));
}

async function refreshCycleShortcut(): Promise<void> {
	const commands = (await chrome.commands?.getAll?.()) ?? [];
	const command = commands.find((c) => c.name === CYCLE_LANGUAGE_COMMAND);
	cycleShortcut = command?.shortcut ?? '';
	cycleShortcutLoaded = true;
}

/** Keys the WebExtension `commands` API accepts, by `KeyboardEvent.code`. */
function commandKey(code: string): string | null {
	if (/^Key[A-Z]$/.test(code)) return code.slice(3);
	if (/^Digit[0-9]$/.test(code)) return code.slice(5);
	if (/^F([1-9]|1[0-2])$/.test(code)) return code;

	const named: Record<string, string> = {
		Space: 'Space',
		Comma: 'Comma',
		Period: 'Period',
		Home: 'Home',
		End: 'End',
		PageUp: 'PageUp',
		PageDown: 'PageDown',
		Insert: 'Insert',
		Delete: 'Delete',
		ArrowUp: 'Up',
		ArrowDown: 'Down',
		ArrowLeft: 'Left',
		ArrowRight: 'Right',
	};
	return named[code] ?? null;
}

function startCycleShortcutCapture(): void {
	capturingCycleShortcut = true;
	cycleShortcutError = '';

	const handleKeydown = async (event: KeyboardEvent) => {
		event.preventDefault();

		if (event.key === 'Escape') {
			stop();
			return;
		}

		const key = commandKey(event.code);
		if (key == null) {
			return; // a modifier on its own, or a key the browser does not allow
		}

		if (event.getModifierState('AltGraph')) {
			cycleShortcutError = 'AltGr types characters, use Ctrl or Alt instead.';
			return;
		}

		const modifiers: string[] = [];
		if (event.ctrlKey) modifiers.push('Ctrl');
		if (event.altKey) modifiers.push('Alt');
		if (event.shiftKey) modifiers.push('Shift');

		if (!event.ctrlKey && !event.altKey) {
			cycleShortcutError = 'The shortcut needs Ctrl or Alt.';
			return;
		}

		try {
			await commandsApi.update?.({
				name: CYCLE_LANGUAGE_COMMAND,
				shortcut: [...modifiers, key].join('+'),
			});
			stop();
		} catch (error) {
			cycleShortcutError = error instanceof Error ? error.message : String(error);
		}
	};

	const stop = () => {
		window.removeEventListener('keydown', handleKeydown, true);
		capturingCycleShortcut = false;
		refreshCycleShortcut();
	};

	window.addEventListener('keydown', handleKeydown, true);
}

async function resetCycleShortcut(): Promise<void> {
	cycleShortcutError = '';
	await commandsApi.update?.({ name: CYCLE_LANGUAGE_COMMAND, shortcut: DEFAULT_CYCLE_SHORTCUT });
	await refreshCycleShortcut();
}

function openBrowserShortcuts(): void {
	chrome.tabs.create({ url: 'chrome://extensions/shortcuts' });
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

// Import removed
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
        <div class="flex items-start justify-between gap-4">
          <div class="flex flex-col">
            <h3 class="text-sm">Languages</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              The keyboard shortcut switches between these languages, in this order.
            </p>
          </div>
          <div class="flex w-72 flex-col gap-2">
            <ol class="space-y-1" data-testid="language-cycle">
              {#each languageCycle as dialect, index (dialect)}
                {@const info = dialectInfo(dialectCatalog, dialect)}
                <li class="flex items-center justify-between gap-2 text-sm">
                  <span>{codeFlag(info?.code ?? '')} {languageLabel(info)}</span>
                  <span class="flex gap-1">
                    <Button size="sm" color="light" title="Move up" disabled={index === 0} on:click={() => moveLanguage(index, -1)}>↑</Button>
                    <Button size="sm" color="light" title="Move down" disabled={index === languageCycle.length - 1} on:click={() => moveLanguage(index, 1)}>↓</Button>
                    <Button size="sm" color="light" title="Remove" disabled={languageCycle.length <= 1} on:click={() => removeLanguage(index)}>✕</Button>
                  </span>
                </li>
              {/each}
            </ol>
            {#if languageCycle.length < MAX_LANGUAGE_CYCLE}
              <Select size="sm" bind:value={languageToAdd} on:change={addLanguage} data-testid="language-add">
                <option value="">Add a language…</option>
                {#each groupByLanguage(availableLanguages) as [language, dialects] (language)}
                  <optgroup label={language}>
                    {#each dialects as info (info.dialect)}
                      <option value={info.dialect}>{codeFlag(info.code)} {info.region}</option>
                    {/each}
                  </optgroup>
                {/each}
              </Select>
            {:else}
              <p class="text-xs text-gray-600 dark:text-gray-400">
                At most {MAX_LANGUAGE_CYCLE} languages.
              </p>
            {/if}
          </div>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between gap-4">
          <div class="flex flex-col">
            <h3 class="text-sm">Switch Language Shortcut</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Switches to the next language in the list above. If it does nothing, another
              extension probably uses the same shortcut.
              {#if canOpenShortcutSettings}
                Manage shortcuts shows the ones used more than once.
              {/if}
            </p>
            {#if cycleShortcutLoaded && !cycleShortcut && !capturingCycleShortcut}
              <p class="text-xs text-red-600" data-testid="cycle-shortcut-missing">
                No shortcut is assigned. The browser leaves it out when another extension already
                uses it, so choose a different one.
              </p>
            {/if}
            {#if cycleShortcutError}
              <p class="text-xs text-red-600">{cycleShortcutError}</p>
            {/if}
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm" data-testid="cycle-shortcut">
              {capturingCycleShortcut ? 'Press the new shortcut (Esc cancels)' : cycleShortcut || 'Not set'}
            </span>
            {#if cycleShortcutEditable}
              <Button size="sm" color="light" disabled={capturingCycleShortcut} on:click={startCycleShortcutCapture}>Change</Button>
              <Button size="sm" color="light" disabled={capturingCycleShortcut} on:click={resetCycleShortcut}>Default</Button>
              {#if canOpenShortcutSettings}
                <Button size="sm" color="light" disabled={capturingCycleShortcut} on:click={() => commandsApi.openShortcutSettings?.()}>Manage shortcuts</Button>
              {/if}
            {:else}
              <Button size="sm" color="light" on:click={openBrowserShortcuts}>Change</Button>
            {/if}
          </div>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">Remember the Last Language</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Keep the language you switched to last after the browser restarts. When off, Harper
              starts with the first language in the list.
            </p>
          </div>
          <input
            type="checkbox"
            checked={rememberLanguage}
            onchange={setRememberLanguageFromCheckbox}
            class="h-5 w-5"
            data-testid="remember-language"
          />
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
            <h3 class="text-sm">Export Enabled Domains</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              Downloads JSON of domains explicitly enabled.
            </p>
          </div>
          <Button size="sm" on:click={exportEnabledDomainsCSV}
            >Export JSON</Button
          >
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

<script lang="ts">
import { Button, Card, Input, Select, Textarea } from 'components';
import {
	Dialect,
	type LintConfig,
	type StructuredLintConfig,
	type StructuredLintSetting,
} from 'harper.js';
import { tick } from 'svelte';
import logo from '/logo.png';
import ProtocolClient from '../ProtocolClient';
import type { Hotkey, Modifier, WeirpackMeta } from '../protocol';
import { ActivationKey } from '../protocol';
import StructuredRuleSettings from './StructuredRuleSettings.svelte';

let loaded = $state(false);

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
let hotkey: Hotkey = $state({ modifiers: ['Ctrl'], key: 'E' });
let buttonText = $state('');
let anyRulesEnabled = $derived(Object.values(lintConfig ?? {}).some((value) => value !== false));
let totalRules = $derived(Object.keys(lintConfig ?? {}).length);
let customizedRules = $derived(
	Object.values(lintConfig ?? {}).filter((value) => value !== null && value !== undefined).length,
);
let weirpacks: WeirpackMeta[] = $state([]);
let weirpackBusy = $state(false);
let weirpackError = $state('');
let fileInputRef: HTMLInputElement | undefined = $state();
/** Announced to screen readers through a polite live region. */
let statusMessage = $state('');

$effect(() => {
	if (!loaded) return;
	ProtocolClient.setLintConfig($state.snapshot(lintConfig));
});

$effect(() => {
	if (!loaded) return;
	ProtocolClient.setDialect(dialect);
});

$effect(() => {
	if (!loaded || !delayLoaded) return;
	ProtocolClient.setDelay(delay);
});

$effect(() => {
	if (!loaded) return;
	ProtocolClient.setDefaultEnabled(defaultEnabled);
});

$effect(() => {
	if (!loaded) return;
	ProtocolClient.setActivationKey(activationKey);
});

$effect(() => {
	if (!loaded) return;
	ProtocolClient.setUserDictionary(stringToDict(userDict));
});

// Load all initial state before displaying settings to prevent flash of defaults
Promise.all([
	ProtocolClient.getLintConfig(),
	ProtocolClient.getStructuredLintConfig(),
	ProtocolClient.getLintDescriptions(),
	ProtocolClient.getDialect(),
	ProtocolClient.getIsolateEnglish(),
	ProtocolClient.getDelay(),
	ProtocolClient.getDefaultEnabled(),
	ProtocolClient.getActivationKey(),
	ProtocolClient.getHotkey(),
	ProtocolClient.getUserDictionary(),
	ProtocolClient.getWeirpacks(),
]).then(
	([
		nextLintConfig,
		nextStructuredConfig,
		nextLintDescriptions,
		nextDialect,
		nextIsolateEnglish,
		nextDelay,
		nextDefaultEnabled,
		nextActivationKey,
		nextHotkey,
		nextUserDict,
		nextWeirpacks,
	]) => {
		lintConfig = nextLintConfig;
		structuredLintConfig = nextStructuredConfig;
		lintDescriptions = nextLintDescriptions;
		dialect = nextDialect;
		isolateEnglish = nextIsolateEnglish;
		delay = nextDelay;
		delayLoaded = true;
		defaultEnabled = nextDefaultEnabled;
		activationKey = nextActivationKey;
		hotkey = {
			modifiers: [...nextHotkey.modifiers],
			key: nextHotkey.key,
		};
		buttonText = `${nextHotkey.modifiers.join('+')} + ${nextHotkey.key.toUpperCase()}`;
		userDict = dictToString(nextUserDict.toSorted());
		weirpacks = nextWeirpacks.toSorted((a, b) => b.installedAt.localeCompare(a.installedAt));

		loaded = true;
	},
);

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
	const enable = !anyRulesEnabled;
	updateAllRules(enable);
	statusMessage = enable ? 'All rules enabled.' : 'All rules disabled.';
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

function setIsolateEnglish(value: boolean): void {
	if (isolateEnglish === value) return;
	isolateEnglish = value;
	ProtocolClient.setIsolateEnglish(isolateEnglish);
}

/** Arrow-key support for the On/Off radio groups (On is the first option). */
async function handleRadioKeydown(event: KeyboardEvent, set: (value: boolean) => void) {
	const previous = ['ArrowLeft', 'ArrowUp'];
	const next = ['ArrowRight', 'ArrowDown'];
	if (!previous.includes(event.key) && !next.includes(event.key)) return;

	event.preventDefault();
	const group = event.currentTarget as HTMLElement;
	set(previous.includes(event.key));
	await tick();
	group.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
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

let isCapturingHotkey = $state(false);
let stopHotkeyCapture: (() => void) | null = null;

function formatHotkey(value: Hotkey): string {
	const key = value.key.length === 1 ? value.key.toUpperCase() : value.key;
	return `${value.modifiers.join('+')} + ${key}`;
}

function cancelHotkeyCapture() {
	stopHotkeyCapture?.();
	stopHotkeyCapture = null;
	isCapturingHotkey = false;
	buttonText = formatHotkey(hotkey);
	statusMessage = 'Hotkey change cancelled.';
}

function startHotkeyCapture() {
	isCapturingHotkey = true;
	buttonText = 'Listening...';
	statusMessage = 'Press the new key combination, or Escape to cancel.';

	const handleKeydown = (event: KeyboardEvent) => {
		// Never trap keyboard users: Escape cancels, a plain Tab moves on.
		if (event.key === 'Escape') {
			event.preventDefault();
			cancelHotkeyCapture();
			return;
		}
		if (event.key === 'Tab' && !event.ctrlKey && !event.altKey) {
			cancelHotkeyCapture();
			return;
		}

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
			const formattedKey = key.length === 1 ? key.toUpperCase() : key;
			buttonText = `${modifiers.join('+')} + ${formattedKey}`;
			const newHotkey = {
				modifiers: [...modifiers],
				key: formattedKey,
			};

			hotkey = newHotkey;
			ProtocolClient.setHotkey(newHotkey);
			window.removeEventListener('keydown', handleKeydown);
			stopHotkeyCapture = null;
			isCapturingHotkey = false;
			statusMessage = `Hotkey set to ${buttonText}.`;
		}
	};

	window.addEventListener('keydown', handleKeydown);
	stopHotkeyCapture = () => window.removeEventListener('keydown', handleKeydown);
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
		statusMessage = 'Weirpack uploaded.';
	} catch (error) {
		const message = error instanceof Error ? error.message : 'Failed to upload Weirpack.';
		weirpackError = message;
	} finally {
		weirpackBusy = false;
		if (input) input.value = '';
	}
}

async function removeWeirpack(id: string) {
	weirpackBusy = true;
	weirpackError = '';
	try {
		await ProtocolClient.removeWeirpack(id);
		await refreshWeirpacks();
		statusMessage = 'Weirpack removed.';
	} catch (error) {
		const message = error instanceof Error ? error.message : 'Failed to remove Weirpack.';
		weirpackError = message;
	} finally {
		weirpackBusy = false;
	}
}
</script>

<main class="flex min-h-screen flex-col px-4 py-6 font-sans lg:h-screen lg:overflow-hidden lg:px-8">
  <!-- Skip links: visible only while focused -->
  <a href="#rules-list" class="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-50 focus:rounded-md focus:bg-primary focus:px-3 focus:py-2 focus:text-sm focus:font-semibold focus:text-black">Skip to rules</a>
  <a href="#settings" class="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-50 focus:rounded-md focus:bg-primary focus:px-3 focus:py-2 focus:text-sm focus:font-semibold focus:text-black">Skip to settings</a>
  <!-- Screen reader announcements -->
  <p class="sr-only" role="status" aria-live="polite" aria-atomic="true">{statusMessage}</p>

  <div class="mx-auto flex min-h-0 w-full max-w-1600 flex-1 flex-col gap-6">
    <!-- Header -->
    <Card class="flex shrink-0 items-center gap-3.5 p-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gray-100 dark:bg-slate-800">
        <img src={logo} alt="Harper logo" class="h-6 w-auto" />
      </div>
      <div class="flex flex-col">
        <h1 class="text-base font-semibold tracking-tight">Harper</h1>
        <p class="text-xs text-gray-500 dark:text-[#d1d5db]!">Extension Settings</p>
      </div>
    </Card>

    {#if loaded}
      <div class="grid min-h-0 flex-1 gap-6 lg:grid-cols-[minmax(0,7fr)_minmax(0,5fr)] lg:grid-rows-[minmax(0,1fr)]">

        <!-- ══ LEFT COLUMN: Rules (sticky, fills viewport height) ══ -->
        <Card class="flex min-h-0 min-w-0 max-h-[75vh] flex-col gap-4 p-6 lg:max-h-none">
          <div class="flex shrink-0 items-baseline justify-between gap-3">
            <h2 class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-[#d1d5db]!">Rules</h2>
            <p class="font-mono text-xs text-gray-600 dark:text-[#e5e7eb]!">
              {totalRules} total · {customizedRules} customized
            </p>
          </div>

          <div class="flex shrink-0 flex-wrap items-center gap-2.5">
            <Input
              bind:value={searchQuery}
              id="rule-search"
              aria-label="Search rules"
              aria-controls="rules-list"
              autocomplete="off"
              placeholder="Search rules..."
              size="sm"
              class="min-w-12rem flex-1 outline-none! focus:border-primary! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
            />
            <Button size="sm" class="cursor-pointer outline-none! focus:outline-none! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!" on:click={() => { resetRulesToDefaults(); statusMessage = 'All rules reset to defaults.'; }}>Reset to Defaults</Button>
            <Button size="sm" color="light" class="cursor-pointer outline-none! focus:outline-none! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!" on:click={toggleAllRules}>
              {anyRulesEnabled ? 'Disable All Rules' : 'Enable All Rules'}
            </Button>
          </div>

          <!-- svelte-ignore a11y_no_noninteractive_tabindex (scrollable region must be keyboard-focusable) -->
          <div
            id="rules-list"
            role="region"
            aria-label="Rules"
            tabindex="0"
            class="rule-scroll min-h-0 flex-1 space-y-4 overflow-y-auto rounded-md pr-1 [scrollbar-width:thin] focus-visible:outline focus-visible:outline-offset-2 focus-visible:outline-primary"
          >
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

        <!-- ══ RIGHT COLUMN: General + Weirpacks ══ -->
        <div id="settings" tabindex="-1" aria-label="Extension settings" role="region" class="min-h-0 min-w-0 space-y-6 outline-none! lg:overflow-y-auto lg:pr-1 [scrollbar-width:thin]">

          <!-- ── GENERAL ───────────────────────────── -->
          <Card class="space-y-6 p-6">
            <h2 class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-[#d1d5db]!">General</h2>

            <div class="divide-y divide-gray-100 dark:divide-slate-800/60">
              <!-- English Dialect -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="dialect-label" class="text-base font-semibold">English Dialect</h3>
                  <p id="dialect-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Select target spelling and grammar rules.</p>
                </div>
                <Select
                  size="sm"
                  class="h-9 w-48 shrink-0 outline-none! focus:border-primary! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                  bind:value={dialect}
              aria-labelledby="dialect-label" aria-describedby="dialect-desc"
                >
                  <option value={Dialect.American}>American English</option>
                  <option value={Dialect.British}>British English</option>
                  <option value={Dialect.Australian}>Australian English</option>
                  <option value={Dialect.Canadian}>Canadian English</option>
                  <option value={Dialect.Indian}>Indian English</option>
                </Select>
              </div>

              <!-- Ignore Non-English Text -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="isolate-label" class="text-base font-semibold">Ignore Non-English Text</h3>
                  <p id="isolate-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Skip text that Harper detects as not English.</p>
                </div>

                <div class="h-9 w-48 shrink-0 rounded-lg border border-gray-200 p-1 dark:border-slate-700 dark:bg-slate-900">
                  <div
                    class="relative flex h-full w-full"
                    role="radiogroup"
                    aria-labelledby="isolate-label" aria-describedby="isolate-desc"
                  >
                    <div
                      class="absolute inset-y-0 left-0 w-1/2 rounded-md bg-primary transition-transform duration-300 ease-out motion-reduce:transition-none"
                      class:translate-x-0={isolateEnglish}
                      class:translate-x-full={!isolateEnglish}
                    ></div>

                    <button
                      type="button"
                      role="radio"
                      aria-checked={isolateEnglish}
                      tabindex={isolateEnglish ? 0 : -1}
                      onkeydown={(e) => handleRadioKeydown(e, setIsolateEnglish)}
                      class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 motion-reduce:transition-none {isolateEnglish ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-[#d1d5db]! dark:hover:text-white!'}"
                      onclick={() => setIsolateEnglish(true)}
                    >
                      On
                    </button>
                    <button
                      type="button"
                      role="radio"
                      aria-checked={!isolateEnglish}
                      tabindex={!isolateEnglish ? 0 : -1}
                      onkeydown={(e) => handleRadioKeydown(e, setIsolateEnglish)}
                      class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 motion-reduce:transition-none {!isolateEnglish ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-[#d1d5db]! dark:hover:text-white!'}"
                      onclick={() => setIsolateEnglish(false)}
                    >
                      Off
                    </button>
                  </div>
                </div>
              </div>

              <!-- Enable on New Sites by Default -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="default-enabled-label" class="text-base font-semibold">Enable on New Sites by Default</h3>
                  <p id="default-enabled-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Automatically run Harper on newly visited websites.</p>
                </div>

                <div class="h-9 w-48 shrink-0 rounded-lg border border-gray-200 p-1 dark:border-slate-700 dark:bg-slate-900">
                  <div
                    class="relative flex h-full w-full"
                    role="radiogroup"
                    aria-labelledby="default-enabled-label" aria-describedby="default-enabled-desc"
                  >
                    <div
                      class="absolute inset-y-0 left-0 w-1/2 rounded-md bg-primary transition-transform duration-300 ease-out motion-reduce:transition-none"
                      class:translate-x-0={defaultEnabled}
                      class:translate-x-full={!defaultEnabled}
                    ></div>

                    <button
                      type="button"
                      role="radio"
                      aria-checked={defaultEnabled}
                      tabindex={defaultEnabled ? 0 : -1}
                      onkeydown={(e) => handleRadioKeydown(e, (v) => (defaultEnabled = v))}
                      class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 motion-reduce:transition-none {defaultEnabled ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-[#d1d5db]! dark:hover:text-white!'}"
                      onclick={() => (defaultEnabled = true)}
                    >
                      On
                    </button>
                    <button
                      type="button"
                      role="radio"
                      aria-checked={!defaultEnabled}
                      tabindex={!defaultEnabled ? 0 : -1}
                      onkeydown={(e) => handleRadioKeydown(e, (v) => (defaultEnabled = v))}
                      class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 motion-reduce:transition-none {!defaultEnabled ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-[#d1d5db]! dark:hover:text-white!'}"
                      onclick={() => (defaultEnabled = false)}
                    >
                      Off
                    </button>
                  </div>
                </div>
              </div>

              <!-- Delay -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="delay-label" class="text-base font-semibold">Delay (ms)</h3>
                  <p id="delay-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Wait time after typing stops before refreshing highlights.</p>
                </div>
                <Input
                  type="number"
                  min="0"
                  step="50"
                  bind:value={delay}
              aria-labelledby="delay-label" aria-describedby="delay-desc"
                  class="h-9 w-48 shrink-0 text-sm font-semibold dark:text-white! outline-none! focus:border-primary! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                />
              </div>

              <!-- Export Enabled Domains -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="export-label" class="text-base font-semibold">Export Enabled Domains</h3>
                  <p id="export-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Downloads a JSON list of domains explicitly enabled.</p>
                </div>
                <Button size="sm" class="h-9 w-48 shrink-0 cursor-pointer text-xs font-semibold outline-none! focus:outline-none! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!" aria-describedby="export-desc" on:click={exportEnabledDomainsCSV}>Export JSON</Button>
              </div>

              <!-- Activation Key -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="activation-label" class="text-base font-semibold">Activation Key</h3>
                  <p id="activation-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Requires a quick double-press before activating highlights.</p>
                </div>
                <Select
                  size="sm"
                  class="h-9 w-48 shrink-0 outline-none! focus:border-primary! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                  bind:value={activationKey}
              aria-labelledby="activation-label" aria-describedby="activation-desc"
                >
                  <option value={ActivationKey.Shift}>Double Shift</option>
                  <option value={ActivationKey.Control}>Double Control</option>
                  <option value={ActivationKey.Off}>Off</option>
                </Select>
              </div>

              <!-- Apply Last Suggestion Hotkey -->
              <div class="flex items-center justify-between gap-4 py-3.5">
                <div class="flex flex-col">
                  <h3 id="hotkey-label" class="text-base font-semibold">Apply Last Suggestion Hotkey</h3>
                  <p id="hotkey-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Applies suggestions to the last highlighted word.</p>
                </div>
                <div class="flex h-9 w-48 shrink-0 items-center justify-between gap-2">
                  <kbd aria-labelledby="hotkey-label" class="flex h-full flex-1 items-center justify-center whitespace-nowrap rounded-md border border-gray-200 bg-gray-100 px-2 font-mono text-xs font-semibold text-gray-800 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100">
                    {buttonText}
                  </kbd>
                  <Button
                    size="sm"
                    class="h-full shrink-0 cursor-pointer text-xs font-semibold outline-none! focus:outline-none! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                    aria-label={isCapturingHotkey ? 'Cancel changing hotkey' : `Modify hotkey, currently ${buttonText}`}
                    on:click={isCapturingHotkey ? cancelHotkeyCapture : startHotkeyCapture}
                  >
                    {isCapturingHotkey ? 'Cancel' : 'Modify'}
                  </Button>
                </div>
              </div>

              <!-- User Dictionary -->
              <div class="flex flex-col gap-2 py-3.5">
                <div>
                  <h3 id="dictionary-label" class="text-base font-semibold">User Dictionary</h3>
                  <p id="dictionary-desc" class="text-xs text-gray-600 dark:text-[#d1d5db]!">Add custom words to ignore (one word per line).</p>
                </div>
                <Textarea
                  bind:value={userDict}
              aria-labelledby="dictionary-label" aria-describedby="dictionary-desc"
                  rows={3}
                  placeholder="customword&#10;anotherterm"
                  class="w-full resize-y text-xs outline-none! focus:border-primary! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                ></Textarea>
              </div>
            </div>
          </Card>

          <!-- ── WEIRPACKS ───────────────────────────── -->
          <Card class="space-y-4 p-6">
            <h2 class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-[#d1d5db]!">Weirpacks</h2>

            <div class="flex items-center justify-between gap-4">
              <p class="text-xs text-gray-600 dark:text-[#d1d5db]!">
                Upload custom <code>.weirpack</code> rule packs.
                <a href="https://writewithharper.com/docs/weir#Weirpacks" target="_blank" rel="noopener" class="text-primary hover:underline">
                  What is a Weirpack?
                </a>
              </p>

              <input
                type="file"
                accept=".weirpack,application/zip"
                multiple
                disabled={weirpackBusy}
                bind:this={fileInputRef}
                onchange={handleWeirpackUpload}
                class="hidden"
              />

              <Button
                size="sm"
                class="h-9 w-48 shrink-0 cursor-pointer text-xs font-semibold outline-none! focus:outline-none! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                disabled={weirpackBusy}
                on:click={() => fileInputRef?.click()}
              >
                {weirpackBusy ? 'Uploading...' : 'Browse Weirpacks'}
              </Button>
            </div>

            {#if weirpackError}
              <p role="alert" class="text-xs text-red-600 dark:text-red-300!">{weirpackError}</p>
            {/if}

            {#if weirpacks.length === 0}
              <p class="text-xs text-gray-500 dark:text-[#d1d5db]!">No custom Weirpacks installed.</p>
            {:else}
              <div class="space-y-2.5">
                {#each weirpacks as weirpack}
                  <div class="flex items-center justify-between gap-3 rounded-lg border border-gray-200 p-3 dark:border-slate-800 dark:bg-slate-800/40">
                    <div class="min-w-0">
                      <p class="truncate text-xs font-semibold">
                        {weirpack.name}{weirpack.version ? ` v${weirpack.version}` : ''}
                      </p>
                      <p class="truncate font-mono text-[11px] text-gray-500 dark:text-[#d1d5db]!">
                        {weirpack.filename}
                      </p>
                    </div>
                    <Button
                      size="sm"
                      color="light"
                      class="cursor-pointer outline-none! focus:outline-none! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!"
                      disabled={weirpackBusy}
                      aria-label={`Remove ${weirpack.name}`}
                      on:click={() => removeWeirpack(weirpack.id)}
                    >
                      Remove
                    </Button>
                  </div>
                {/each}
              </div>
            {/if}
          </Card>
        </div>
      </div>
    {/if}
  </div>
</main>
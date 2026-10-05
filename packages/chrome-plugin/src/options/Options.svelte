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
let weirpacks: WeirpackMeta[] = $state([]);
let weirpackBusy = $state(false);
let weirpackError = $state('');
let fileInputRef: HTMLInputElement | undefined = $state();

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

function toggleIsolateEnglish(): void {
	isolateEnglish = !isolateEnglish;
	ProtocolClient.setIsolateEnglish(isolateEnglish);
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

function startHotkeyCapture() {
	isCapturingHotkey = true;
	buttonText = 'Listening...';

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
			const formattedKey = key.length === 1 ? key.toUpperCase() : key;
			buttonText = `${modifiers.join('+')} + ${formattedKey}`;
			const newHotkey = {
				modifiers: [...modifiers],
				key: formattedKey,
			};

			hotkey = newHotkey;
			ProtocolClient.setHotkey(newHotkey);
			window.removeEventListener('keydown', handleKeydown);
			isCapturingHotkey = false;
		}
	};

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
		if (input) input.value = '';
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

<div class="min-h-screen px-4 py-10 font-sans">
  <div class="mx-auto max-w-screen-lg space-y-6">
    <!-- Header -->
    <Card class="flex items-center gap-3.5 p-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gray-100 dark:bg-slate-800">
        <img src={logo} alt="Harper logo" class="h-6 w-auto" />
      </div>
      <div class="flex flex-col">
        <h1 class="text-base font-semibold tracking-tight">Harper</h1>
        <p class="text-xs text-gray-500 dark:text-slate-400">Extension Settings</p>
      </div>
    </Card>

    {#if loaded}
      <!-- ── GENERAL ───────────────────────────── -->
      <Card class="space-y-6 p-6">
        <h2 class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-slate-400">General</h2>

        <div class="divide-y divide-gray-100 dark:divide-slate-800/60">
          <!-- English Dialect -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">English Dialect</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Select target spelling and grammar rules.</p>
            </div>
            <Select
              size="sm"
              class="h-9 w-48 !outline-none focus:!border-primary focus:!ring-0"
              bind:value={dialect}
            >
              <option value={Dialect.American}>American English</option>
              <option value={Dialect.British}>British English</option>
              <option value={Dialect.Australian}>Australian English</option>
              <option value={Dialect.Canadian}>Canadian English</option>
              <option value={Dialect.Indian}>Indian English</option>
            </Select>
          </div>

          <!-- Ignore Non-English Text -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">Ignore Non-English Text</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Skip text that Harper detects as not English.</p>
            </div>

            <div class="h-9 w-48 shrink-0 rounded-lg border border-gray-200 p-1 dark:border-slate-700 dark:bg-slate-900">
              <div
                class="relative flex h-full w-full"
                role="radiogroup"
                aria-label="Ignore Non-English Text"
              >
                <div
                  class="absolute inset-y-0 left-0 w-1/2 rounded-md bg-primary transition-transform duration-300 ease-out"
                  class:translate-x-0={isolateEnglish}
                  class:translate-x-full={!isolateEnglish}
                ></div>

                <button
                  type="button"
                  role="radio"
                  aria-checked={isolateEnglish}
                  class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 {isolateEnglish ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-slate-400 dark:hover:text-white'}"
                  onclick={toggleIsolateEnglish}
                >
                  On
                </button>
                <button
                  type="button"
                  role="radio"
                  aria-checked={!isolateEnglish}
                  class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 {!isolateEnglish ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-slate-400 dark:hover:text-white'}"
                  onclick={toggleIsolateEnglish}
                >
                  Off
                </button>
              </div>
            </div>
          </div>

          <!-- Enable on New Sites by Default -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">Enable on New Sites by Default</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Automatically run Harper on newly visited websites.</p>
            </div>

            <div class="h-9 w-48 shrink-0 rounded-lg border border-gray-200 p-1 dark:border-slate-700 dark:bg-slate-900">
              <div
                class="relative flex h-full w-full"
                role="radiogroup"
                aria-label="Enable on New Sites by Default"
              >
                <div
                  class="absolute inset-y-0 left-0 w-1/2 rounded-md bg-primary transition-transform duration-300 ease-out"
                  class:translate-x-0={defaultEnabled}
                  class:translate-x-full={!defaultEnabled}
                ></div>

                <button
                  type="button"
                  role="radio"
                  aria-checked={defaultEnabled}
                  class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 {defaultEnabled ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-slate-400 dark:hover:text-white'}"
                  onclick={() => (defaultEnabled = true)}
                >
                  On
                </button>
                <button
                  type="button"
                  role="radio"
                  aria-checked={!defaultEnabled}
                  class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-xs font-semibold transition-colors duration-200 {!defaultEnabled ? 'text-black' : 'text-gray-500 hover:text-gray-900 dark:text-slate-400 dark:hover:text-white'}"
                  onclick={() => (defaultEnabled = false)}
                >
                  Off
                </button>
              </div>
            </div>
          </div>

          <!-- Delay -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">Delay (ms)</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Wait time after typing stops before refreshing highlights.</p>
            </div>
            <Input
              type="number"
              min="0"
              step="50"
              bind:value={delay}
              class="h-9 w-48 text-xs !outline-none focus:!border-primary focus:!ring-0"
            />
          </div>

          <!-- Export Enabled Domains -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">Export Enabled Domains</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Downloads a JSON list of domains explicitly enabled.</p>
            </div>
            <Button size="sm" class="h-9 w-48 cursor-pointer text-xs font-semibold !outline-none focus:!outline-none focus:!ring-0" on:click={exportEnabledDomainsCSV}>Export JSON</Button>
          </div>

          <!-- Activation Key -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">Activation Key</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Requires a quick double-press before activating highlights.</p>
            </div>
            <Select
              size="sm"
              class="h-9 w-48 !outline-none focus:!border-primary focus:!ring-0"
              bind:value={activationKey}
            >
              <option value={ActivationKey.Shift}>Double Shift</option>
              <option value={ActivationKey.Control}>Double Control</option>
              <option value={ActivationKey.Off}>Off</option>
            </Select>
          </div>

          <!-- Apply Last Suggestion Hotkey -->
          <div class="flex items-center justify-between py-3.5">
            <div class="flex flex-col">
              <h3 class="text-base font-semibold">Apply Last Suggestion Hotkey</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Applies suggestions to the last highlighted word.</p>
            </div>
            <div class="flex h-9 w-48 items-center justify-between gap-2">
              <kbd class="flex h-full flex-1 items-center justify-center whitespace-nowrap rounded-md border border-gray-200 bg-gray-100 px-2 font-mono text-xs font-semibold text-gray-800 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100">
                {buttonText}
              </kbd>
              <Button
                size="sm"
                class="h-full shrink-0 cursor-pointer text-xs font-semibold !outline-none focus:!outline-none focus:!ring-0"
                on:click={startHotkeyCapture}
              >
                {isCapturingHotkey ? 'Cancel' : 'Modify'}
              </Button>
            </div>
          </div>

          <!-- User Dictionary -->
          <div class="flex flex-col gap-2 py-3.5">
            <div>
              <h3 class="text-base font-semibold">User Dictionary</h3>
              <p class="text-xs text-gray-600 dark:text-gray-400">Add custom words to ignore (one word per line).</p>
            </div>
            <Textarea
              bind:value={userDict}
              rows={3}
              placeholder="customword&#10;anotherterm"
              class="w-full resize-y text-xs !outline-none focus:!border-primary focus:!ring-0"
            ></Textarea>
          </div>
        </div>
      </Card>

      <!-- ── WEIRPACKS ───────────────────────────── -->
      <Card class="space-y-4 p-6">
        <h2 class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-slate-400">Weirpacks</h2>

        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <p class="text-xs text-gray-600 dark:text-slate-400">
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
            class="h-9 w-48 shrink-0 cursor-pointer text-xs font-semibold !outline-none focus:!outline-none focus:!ring-0"
            disabled={weirpackBusy}
            on:click={() => fileInputRef?.click()}
          >
            {weirpackBusy ? 'Uploading...' : 'Browse Weirpacks'}
          </Button>
        </div>

        {#if weirpackError}
          <p class="text-xs text-red-500 dark:text-red-400">{weirpackError}</p>
        {/if}

        {#if weirpacks.length === 0}
          <p class="text-xs text-gray-500 dark:text-slate-400">No custom Weirpacks installed.</p>
        {:else}
          <div class="space-y-2.5">
            {#each weirpacks as weirpack}
              <div class="flex items-center justify-between gap-3 rounded-lg border border-gray-200 p-3 dark:border-slate-800 dark:bg-slate-800/40">
                <div class="min-w-0">
                  <p class="truncate text-xs font-semibold">
                    {weirpack.name}{weirpack.version ? ` v${weirpack.version}` : ''}
                  </p>
                  <p class="truncate font-mono text-[11px] text-gray-500 dark:text-slate-400">
                    {weirpack.filename}
                  </p>
                </div>
                <Button
                  size="sm"
                  color="light"
                  class="cursor-pointer !outline-none focus:!outline-none focus:!ring-0"
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
      <Card class="space-y-4 p-6">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <h2 class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-slate-400">Rules</h2>
          <Input
            bind:value={searchQuery}
            placeholder="Search rules..."
            size="sm"
            class="w-full sm:w-64 !outline-none focus:!border-primary focus:!ring-0"
          />
        </div>

        <div class="flex flex-wrap gap-2.5 pt-1">
          <Button size="sm" class="cursor-pointer !outline-none focus:!outline-none focus:!ring-0" on:click={resetRulesToDefaults}>Reset to Defaults</Button>
          <Button size="sm" color="light" class="cursor-pointer !outline-none focus:!outline-none focus:!ring-0" on:click={toggleAllRules}>
            {anyRulesEnabled ? 'Disable All Rules' : 'Enable All Rules'}
          </Button>
        </div>

        <div class="rule-scroll max-h-96 space-y-4 overflow-y-auto pr-1 [scrollbar-width:thin]">
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
    {/if}
  </div>
</div>
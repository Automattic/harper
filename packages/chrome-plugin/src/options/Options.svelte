<script lang="ts">
import { Button, Card, Input, Select, Textarea } from 'components';
import { Dialect, type LintConfig } from 'harper.js';
import logo from '/logo.png';
import ProtocolClient from '../ProtocolClient';
import type { Hotkey, Modifier, WeirpackMeta } from '../protocol';
import { ActivationKey } from '../protocol';

let lintConfig: LintConfig = $state({});
let lintDescriptions: Record<string, string> = $state({});
let searchQuery = $state('');
let searchQueryLower = $derived(searchQuery.toLowerCase());
let dialect = $state(Dialect.American);
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
	ProtocolClient.setDefaultEnabled(defaultEnabled);
});

$effect(() => {
	ProtocolClient.setActivationKey(activationKey);
});

$effect(() => {
	ProtocolClient.setUserDictionary(stringToDict(userDict));
});

ProtocolClient.getLintConfig().then((l) => {
	lintConfig = l;
});

ProtocolClient.getLintDescriptions().then((d) => {
	lintDescriptions = d;
});

ProtocolClient.getDialect().then((d) => {
	dialect = d;
});

ProtocolClient.getDefaultEnabled().then((d) => {
	defaultEnabled = d;
});

ProtocolClient.getActivationKey().then((d) => {
	activationKey = d;
});

ProtocolClient.getHotkey().then((d) => {
	// Ensure we have a plain object, not a Proxy
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

function configValueToString(value: boolean | undefined): string {
	switch (value) {
		case true:
			return 'enable';
		case false:
			return 'disable';
		case undefined:
		case null:
			return 'default';
	}
}

function configStringToValue(str: string): boolean | undefined | null {
	switch (str) {
		case 'enable':
			return true;
		case 'disable':
			return false;
		case 'default':
			return null;
	}

	throw 'Fell through case';
}

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
        <div class="flex items-center justify-between">
          <h3 class="text-sm">English Dialect</h3>
          <Select
            size="sm"
            class="w-44"
            bind:value={dialect}
          >
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
            <h3 class="text-sm">Enable on New Sites by Default</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">Can make some apps behave abnormally.</p>
          </div>
          <input type="checkbox" bind:checked={defaultEnabled} class="h-5 w-5" />
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">Export Enabled Domains</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">Downloads JSON of domains explicitly enabled.</p>
          </div>
          <Button size="sm" on:click={exportEnabledDomainsCSV}>Export JSON</Button>
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
          <Select
            size="sm"
            class="w-44"
            bind:value={activationKey}
          >
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
            <p class="text-xs text-gray-600 dark:text-gray-400">Applies suggestion to last highlighted word.</p>
          </div>
          <Textarea readonly bind:value={buttonText} />
          <Button size="sm" color="light" style="background-color: {isBlue ? 'blue' : ''}" bind:this={modifyHotkeyButton} on:click={() => {startHotkeyCapture(modifyHotkeyButton); isBlue = !isBlue}}>Modify Hotkey</Button>

        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <h3 class="text-sm">User Dictionary</h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">Each word should be on its own line.</p>
          </div>
          <Textarea
            bind:value={userDict}
          ></Textarea>
        </div>
      </div>

    </Card>

    <Card class="space-y-4">
      <h2 class="pb-1 text-xs uppercase tracking-wider">Weirpacks</h2>

      <div class="space-y-2 flex flex-row w-full justify-between">
        <p class="text-xs text-gray-600 dark:text-gray-400">
          Upload one or more <code>.weirpack</code> files to add custom rule packs.
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
        <p class="text-sm text-gray-600 dark:text-gray-400">No Weirpacks installed.</p>
      {:else}
        <div class="space-y-3">
          {#each weirpacks as weirpack}
            <div class="flex items-center justify-between gap-3 rounded-md border border-primary-100 p-3">
              <div class="min-w-0">
                <p class="truncate text-sm">
                  {weirpack.name}{weirpack.version ? ` v${weirpack.version}` : ''}
                </p>
                <p class="truncate text-xs text-gray-600 dark:text-gray-400">{weirpack.filename}</p>
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
        <Button size="sm" on:click={resetRulesToDefaults}>Reset to Default Rules</Button>
        <Button size="sm" on:click={toggleAllRules}>
          {anyRulesEnabled ? 'Disable All Rules' : 'Enable All Rules'}
        </Button>
      </div>

      {#each Object.entries(lintConfig).filter(
        ([key]) =>
          (lintDescriptions[key] ?? '').toLowerCase().includes(searchQueryLower) ||
          key.toLowerCase().includes(searchQueryLower)
      ) as [key, value]}
        <div class="rule-scroll space-y-4 max-h-80 overflow-y-auto pr-1">
          <!-- rule card sample -->
            <div class="flex items-start justify-between gap-4">
              <div class="space-y-0.5">
                <h3 class="text-sm">{key}</h3>
                <p class="text-xs">{@html lintDescriptions[key]}</p>
              </div>
              <Select
                size="md"
                value={configValueToString(value)}
                on:change={(e) => {
                  lintConfig[key] = configStringToValue(e.target.value);
                }}
              >
                <option value="default">⚙️ Default</option>
                <option value="enable">✅ On</option>
                <option value="disable">🚫 Off</option>
              </Select>
            </div>
          </div>
      {/each}

    </Card>
  </div>
</div>

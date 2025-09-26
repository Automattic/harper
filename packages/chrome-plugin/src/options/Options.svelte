<script lang="ts">
import { Button, Input, Select } from 'flowbite-svelte';
import { Dialect, type LintConfig } from 'harper.js';
import logo from '/logo.png';
import ProtocolClient from '../ProtocolClient';
import { ActivationKey } from '../protocol';
import type { Modifier, Hotkey } from '../protocol';

let lintConfig: LintConfig = $state({});
let lintDescriptions: Record<string, string> = $state({});
let searchQuery = $state('');
let searchQueryLower = $derived(searchQuery.toLowerCase());
let dialect = $state(Dialect.American);
let defaultEnabled = $state(false);
let activationKey: ActivationKey = $state(ActivationKey.Off);
let userDict = $state('');
let modifyHotkeyButton: Button;
let hotkey : Hotkey = $state({modifiers: ['Ctrl'], key: 'e'});

$effect(() => {
	ProtocolClient.setLintConfig(lintConfig);
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
		key: d.key
	};
	buttonText = `Hotkey: ${d.modifiers.join('+')}+${d.key}`;
});


ProtocolClient.getUserDictionary().then((d) => {
	userDict = dictToString(d.toSorted());
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
function startHotkeyCapture(modifyHotkeyButton: Button) {

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
          key: key
        };
        hotkey = newHotkey;
        
        // Call ProtocolClient directly with the plain object to avoid proxy issues
        ProtocolClient.setHotkey(newHotkey);
        
        // Remove listener
        window.removeEventListener('keydown', handleKeydown);

        // change button color
        isBlue = !isBlue;
      }

    }

    // Add temporary key listener
    window.addEventListener('keydown', handleKeydown);


  };



// Import removed
</script>

<!-- centered wrapper with side gutters -->
<div class="mx-auto max-w-screen-md px-4">
  <header class="flex items-center gap-2 px-3 py-2 bg-gray-50/60 border-b border-gray-200 rounded-t-lg">
    <img src={logo} alt="Harper logo" class="h-6 w-auto" />
    <span class="font-semibold text-sm">Harper</span>
  </header>

  <main class="p-6 space-y-10 text-sm text-gray-800 border border-gray-200 rounded-b-lg shadow-sm">
    <!-- ── GENERAL ───────────────────────────── -->
    <section class="space-y-6">
      <h3 class="pb-1 border-b border-gray-200 text-xs font-semibold uppercase tracking-wider text-gray-500">General</h3>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <span class="font-medium">English Dialect</span>
          <Select size="sm" color="primary" class="w-44" bind:value={dialect}>
            <option value={Dialect.American}>🇺🇸 American</option>
            <option value={Dialect.British}>🇬🇧 British</option>
            <option value={Dialect.Australian}>🇦🇺 Australian</option>
            <option value={Dialect.Canadian}>🇨🇦 Canadian</option>
          </Select>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">Enable on New Sites by Default</span>
            <span class="font-light">Can make some apps behave abnormally.</span>
          </div>
          <input type="checkbox" bind:checked={defaultEnabled}/>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">Export Enabled Domains</span>
            <span class="font-light">Downloads JSON of domains explicitly enabled.</span>
          </div>
          <Button size="sm" color="light" on:click={exportEnabledDomainsCSV}>Export JSON</Button>
        </div>
      </div>

      

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">Activation Key</span>
            <span class="font-light">If you're finding that you're accidentally triggering Harper.</span>
          </div>
          <Select size="sm" color="primary" class="w-44" bind:value={activationKey}>
            <option value={ActivationKey.Shift}>Double Shift</option>
            <option value={ActivationKey.Control}>Double Control</option>
            <option value={ActivationKey.Off}>Off</option>
          </Select>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">Apply Last Suggestion Hotkey</span>
            <span class="font-light">Hotkey to apply the most likely suggestion to previously incorrect lint.</span>
          </div>
          <textarea bind:value={buttonText} />
          <Button size="sm" color="light" style="background-color: {isBlue ? 'blue' : ''}" bind:this={modifyHotkeyButton} on:click={() => {startHotkeyCapture(modifyHotkeyButton); isBlue = !isBlue}}>Modify Hotkey</Button>

        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">User Dictionary</span>
            <span class="font-light">Each word should be on its own line.</span>
          </div>
          <textarea bind:value={userDict} />
        </div>
      </div>

    </section>

    <!-- ── RULES ─────────────────────────────── -->
    <section class="space-y-4">
      <div class="flex items-center justify-between gap-4">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500">Rules</h3>
        <Input bind:value={searchQuery} placeholder="Search for a rule…" size="sm" class="w-60" />
      </div>

      {#each Object.entries(lintConfig).filter(([key]) => lintDescriptions[key].toLowerCase().includes(searchQueryLower) || key.toLowerCase().includes(searchQueryLower)) as [key, value]}
        <div class="space-y-4 max-h-80 overflow-y-auto pr-1">
          <!-- rule card sample -->
          <div class="rounded-lg border border-gray-200 p-3 shadow-sm">
            <div class="flex items-start justify-between gap-4">
              <div class="space-y-0.5">
                <p class="font-medium">{key}</p>
                <p class="text-xs text-gray-600">{@html lintDescriptions[key]}</p>
              </div>
              <Select
                size="sm"
                value={configValueToString(value)}
                on:change={(e) => {
                  lintConfig[key] = configStringToValue(e.target.value);
                }}
                class="max-w-[10rem]"
              >
                <option value="default">⚙️ Default</option>
                <option value="enable">✅ On</option>
                <option value="disable">🚫 Off</option>
              </Select>
            </div>
          </div>
        </div>
      {/each}

    </section>
  </main>
</div>

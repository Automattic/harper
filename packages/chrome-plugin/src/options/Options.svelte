<script lang="ts">
import { Button, Card, Input, Select, Textarea } from 'components';
import { Dialect, type LintConfig } from 'harper.js';
import logo from '/logo.png';
import ProtocolClient from '../ProtocolClient';
import { ActivationKey } from '../protocol';

let lintConfig: LintConfig = $state({});
let lintDescriptions: Record<string, string> = $state({});
let searchQuery = $state('');
let searchQueryLower = $derived(searchQuery.toLowerCase());
let dialect = $state(Dialect.American);
let defaultEnabled = $state(false);
let activationKey: ActivationKey = $state(ActivationKey.Off);
let userDict = $state('');
let anyRulesEnabled = $derived(Object.values(lintConfig ?? {}).some((value) => value !== false));

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
	console.log('hit');
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
</script>

<!-- centered wrapper with side gutters -->
<div class="min-h-screen px-4 py-10">
  <div class="mx-auto max-w-screen-lg space-y-4">
    <Card class="flex items-center gap-3">
      <div class="flex h-9 w-9 items-center justify-center rounded-xl">
        <img src={logo} alt="Harper logo" class="h-5 w-auto" />
      </div>
      <div class="flex flex-col">
        <span class="text-sm font-semibold tracking-wide">Harper</span>
        <span class="text-xs">Chrome Extension Settings</span>
      </div>
    </Card>

    <!-- ── GENERAL ───────────────────────────── -->
    <Card class="space-y-6">
      <h3 class="pb-1 text-xs font-semibold uppercase tracking-wider">General</h3>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <span class="font-medium">English Dialect</span>
          <Select
            size="sm"
            class="w-44"
            bind:value={dialect}
          >
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
          <input type="checkbox" bind:checked={defaultEnabled} class="h-5 w-5" />
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">Export Enabled Domains</span>
            <span class="font-light">Downloads JSON of domains explicitly enabled.</span>
          </div>
          <Button size="sm" on:click={exportEnabledDomainsCSV}>Export JSON</Button>
        </div>
      </div>

      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="font-medium">Activation Key</span>
            <span class="font-light">If you're finding that you're accidentally triggering Harper.</span>
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
            <span class="font-medium">User Dictionary</span>
            <span class="font-light">Each word should be on its own line.</span>
          </div>
          <Textarea
            bind:value={userDict}
          ></Textarea>
        </div>
      </div>

    </Card>

    <!-- ── RULES ─────────────────────────────── -->
    <Card class="space-y-4">
      <div class="flex items-center justify-between gap-4">
        <h3 class="text-xs font-semibold uppercase tracking-wider">Rules</h3>
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
          <div class="rounded-xl p-3 shadow-md transition">
            <div class="flex items-start justify-between gap-4">
              <div class="space-y-0.5">
                <p class="font-medium">{key}</p>
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
        </div>
      {/each}

    </Card>
  </div>
</div>

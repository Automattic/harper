<script lang="ts">
import type { LintConfig, StructuredLintConfig, StructuredLintSetting } from 'harper.js';
import { onMount } from 'svelte';
import { Client } from '$lib/client';

type RuleOverride = 'default' | 'on' | 'off';
type RuleItem = { id: string; name: string; desc: string };
type RuleGroup = { id: string; title: string; desc: string; rules: RuleItem[] };
type MatchedRuleGroup = RuleGroup & { matchedRules: RuleItem[] };

const defaultRuleOptions: { value: RuleOverride; label: string }[] = [
	{ value: 'default', label: 'Default' },
	{ value: 'on', label: 'Enabled' },
	{ value: 'off', label: 'Disabled' },
];

let lintConfig: LintConfig | null = null;
let defaultLintConfig: LintConfig | null = null;
let structuredLintConfig: StructuredLintConfig | null = null;
let rules: Record<string, RuleOverride> = {};
let rulesSearch = '';
let expandedGroups: Record<string, boolean> = {};
let isLintConfigLoading = true;
let isLintConfigSaving = false;
let lintConfigError = '';

$: rulesQuery = rulesSearch.trim().toLowerCase();
$: ruleGroups = structuredLintConfig ? ruleGroupsFromStructuredConfig(structuredLintConfig) : [];
$: displayedRules = ruleGroups.flatMap((group) => group.rules);
$: enabledRuleCount = displayedRules.filter((rule) => isRuleEnabled(rule)).length;
$: customizedRuleCount = Object.values(rules).filter((value) => value !== 'default').length;
$: filteredRuleGroups = getFilteredRuleGroups(ruleGroups, rulesQuery);

onMount(() => {
	void loadLintConfig();

	const refreshLintConfig = () => {
		if (!isLintConfigSaving) {
			void loadLintConfig();
		}
	};

	window.addEventListener('focus', refreshLintConfig);

	return () => {
		window.removeEventListener('focus', refreshLintConfig);
	};
});

async function loadLintConfig() {
	isLintConfigLoading = true;
	lintConfigError = '';

	const [fetchedLintConfig, fetchedDefaultLintConfig, fetchedStructuredLintConfig] =
		await Promise.all([
			Client.getLintConfig(),
			Client.getDefaultLintConfig(),
			Client.getStructuredLintConfig(),
		]);

	lintConfig = fetchedLintConfig;
	defaultLintConfig = fetchedDefaultLintConfig;
	structuredLintConfig = fetchedStructuredLintConfig;
	rules = rulesFromLintConfig(fetchedLintConfig, fetchedDefaultLintConfig);
	isLintConfigLoading = false;
}

function rulesFromLintConfig(
	config: LintConfig,
	defaultConfig: LintConfig,
): Record<string, RuleOverride> {
	return Object.fromEntries(
		Object.entries(config).map(([ruleId, value]) => [
			ruleId,
			value === defaultConfig[ruleId] ? 'default' : lintValueToRuleOverride(value),
		]),
	);
}

function lintValueToRuleOverride(value: boolean | null | undefined): RuleOverride {
	if (value === true) return 'on';
	if (value === false) return 'off';
	return 'default';
}

function ruleOverrideToLintValue(ruleId: string, value: RuleOverride): boolean {
	if (value === 'on') return true;
	if (value === 'off') return false;

	return defaultLintConfig?.[ruleId] ?? false;
}

function ruleLabelFromKey(key: string) {
	return key
		.replace(/([a-z0-9])([A-Z])/g, '$1 $2')
		.replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
		.trim();
}

function ruleGroupsFromStructuredConfig(config: StructuredLintConfig): RuleGroup[] {
	const groups: RuleGroup[] = [];
	const looseRules: RuleItem[] = [];

	for (const setting of config.settings) {
		if ('Group' in setting) {
			groups.push(...groupsFromSetting(setting.Group));
		} else {
			looseRules.push(...rulesFromSetting(setting));
		}
	}

	if (looseRules.length > 0) {
		groups.push({
			id: 'ungrouped-rules',
			title: 'Ungrouped Rules',
			desc: "Rules from the app's current lint configuration that are not assigned to a category.",
			rules: looseRules,
		});
	}

	return groups;
}

function groupsFromSetting(
	group: Extract<StructuredLintSetting, { Group: unknown }>['Group'],
): RuleGroup[] {
	const groups: RuleGroup[] = [];
	const rules: RuleItem[] = [];

	for (const setting of group.child.settings) {
		if ('Group' in setting) {
			groups.push(...groupsFromSetting(setting.Group));
		} else {
			rules.push(...rulesFromSetting(setting));
		}
	}

	if (rules.length > 0) {
		groups.unshift({
			id: `${groups.length}-${group.label}`,
			title: group.label,
			desc: group.description,
			rules,
		});
	}

	return groups;
}

function rulesFromSetting(setting: StructuredLintSetting): RuleItem[] {
	if ('Bool' in setting) {
		return [
			{
				id: setting.Bool.name,
				name: setting.Bool.label ?? ruleLabelFromKey(setting.Bool.name),
				desc: 'Harper rule from the curated rule catalog.',
			},
		];
	}

	if ('OneOfMany' in setting) {
		return setting.OneOfMany.names.map((name, index) => ({
			id: name,
			name: setting.OneOfMany.labels?.[index] ?? ruleLabelFromKey(name),
			desc: 'Harper rule option from the curated rule catalog.',
		}));
	}

	return [];
}

async function saveLintConfig(nextLintConfig: LintConfig, nextRules: Record<string, RuleOverride>) {
	const previousLintConfig = lintConfig;
	const previousRules = rules;

	lintConfig = nextLintConfig;
	rules = nextRules;
	isLintConfigSaving = true;
	lintConfigError = '';

	try {
		await Client.setLintConfig(nextLintConfig);
	} catch (error) {
		lintConfig = previousLintConfig;
		rules = previousRules;
		lintConfigError = `Unable to save lint config: ${error}`;
	} finally {
		isLintConfigSaving = false;
	}
}

function setLintConfigRuleValue(config: LintConfig, ruleId: string, value: RuleOverride) {
	config[ruleId] = ruleOverrideToLintValue(ruleId, value);
}

function getFilteredRuleGroups(ruleGroups: RuleGroup[], query: string): MatchedRuleGroup[] {
	if (!query) {
		return ruleGroups.map((group) => ({ ...group, matchedRules: group.rules }));
	}

	return ruleGroups
		.map((group) => {
			const groupMatches =
				group.title.toLowerCase().includes(query) || group.desc.toLowerCase().includes(query);
			const matchedRules = group.rules.filter(
				(rule) =>
					rule.name.toLowerCase().includes(query) || rule.desc.toLowerCase().includes(query),
			);

			if (groupMatches) {
				return { ...group, matchedRules: group.rules };
			}

			if (matchedRules.length > 0) {
				return { ...group, matchedRules };
			}

			return null;
		})
		.filter((group): group is MatchedRuleGroup => group !== null);
}

function getRuleValue(ruleId: string): RuleOverride {
	return rules[ruleId] ?? 'default';
}

function isRuleEnabled(rule: RuleItem) {
	return lintConfig?.[rule.id] ?? defaultLintConfig?.[rule.id] ?? false;
}

async function setRuleOverride(ruleId: string, value: RuleOverride) {
	const nextRules = { ...rules };

	nextRules[ruleId] = value;

	if (!lintConfig) {
		rules = nextRules;
		return;
	}

	const nextLintConfig = { ...lintConfig };
	setLintConfigRuleValue(nextLintConfig, ruleId, value);
	await saveLintConfig(nextLintConfig, nextRules);
}

async function setGroupOverride(group: RuleGroup, value: RuleOverride) {
	const nextRules = { ...rules };
	const nextLintConfig = lintConfig ? { ...lintConfig } : null;

	for (const rule of group.rules) {
		nextRules[rule.id] = value;

		if (nextLintConfig) {
			setLintConfigRuleValue(nextLintConfig, rule.id, value);
		}
	}

	if (!nextLintConfig) {
		rules = nextRules;
		return;
	}

	await saveLintConfig(nextLintConfig, nextRules);
}

function getGroupState(group: RuleGroup): RuleOverride | 'mixed' {
	const values = group.rules.map((rule) => getRuleValue(rule.id));
	const first = values[0];
	return values.every((value) => value === first) ? first : 'mixed';
}

async function resetRules() {
	if (!lintConfig) {
		rules = {};
		return;
	}

	const nextLintConfig = { ...lintConfig };

	for (const rule of displayedRules) {
		nextLintConfig[rule.id] = defaultLintConfig?.[rule.id] ?? false;
	}

	await saveLintConfig(
		nextLintConfig,
		rulesFromLintConfig(nextLintConfig, defaultLintConfig ?? {}),
	);
}

async function disableRules() {
	const nextRules = Object.fromEntries(
		displayedRules.map((rule) => [rule.id, 'off' as RuleOverride]),
	);

	if (!lintConfig) {
		rules = nextRules;
		return;
	}

	const nextLintConfig = { ...lintConfig };

	for (const rule of displayedRules) {
		nextLintConfig[rule.id] = false;
	}

	await saveLintConfig(nextLintConfig, nextRules);
}

function toggleGroup(groupId: string) {
	if (rulesQuery) return;
	expandedGroups = { ...expandedGroups, [groupId]: !expandedGroups[groupId] };
}
</script>

<section>
        <div class="rules-heading">
          <div class="eyebrow">Rules</div>
          <h1>{displayedRules.length} rules, grouped by topic</h1>
          <p>{enabledRuleCount} enabled, {customizedRuleCount} customized.</p>
        </div>

        {#if isLintConfigLoading}
          <p class="result-summary">Loading lint config...</p>
        {:else if lintConfigError}
          <p class="result-summary">{lintConfigError}</p>
        {:else if isLintConfigSaving}
          <p class="result-summary">Saving lint config...</p>
        {/if}

        <div class="sticky-tools">
          <div class="rule-search">
            <span class="settings-icon icon-search" aria-hidden="true"></span>
            <input type="text" placeholder="Search rules..." bind:value={rulesSearch} />
            {#if rulesSearch}
              <button class="icon-button" type="button" aria-label="Clear search" on:click={() => (rulesSearch = "")}>
                <span class="settings-icon icon-x" aria-hidden="true"></span>
              </button>
            {/if}
          </div>
          <button class="button" type="button" disabled={isLintConfigLoading || isLintConfigSaving} on:click={resetRules}>
            Reset to defaults
          </button>
          <button class="button" type="button" disabled={isLintConfigLoading || isLintConfigSaving} on:click={disableRules}>
            Disable all
          </button>
        </div>

        {#if rulesQuery}
          <p class="result-summary">
            {filteredRuleGroups.reduce((total, group) => total + group.matchedRules.length, 0)}
            rules match "{rulesSearch}" across {filteredRuleGroups.length} groups.
          </p>
        {/if}

        <div class="rule-groups">
          {#each filteredRuleGroups as group}
            {@const expanded = rulesQuery || expandedGroups[group.id]}
            {@const groupState = getGroupState(group)}
            <article class="rule-group">
              <button class="group-head" type="button" on:click={() => toggleGroup(group.id)}>
                <svg class:expanded class="chevron" viewBox="0 0 16 16" aria-hidden="true">
                  <path
                    fill-rule="evenodd"
                    d="M6.22 4.22a.75.75 0 0 1 1.06 0l3.25 3.25a.75.75 0 0 1 0 1.06l-3.25 3.25a.75.75 0 0 1-1.06-1.06L8.94 8 6.22 5.28a.75.75 0 0 1 0-1.06Z"
                    clip-rule="evenodd"
                  />
                </svg>
                <span class="grow">
                  <strong>{group.title}</strong>
                  <p>{group.desc}</p>
                  <small>
                    {group.rules.length} rules, {group.rules.filter((rule) => isRuleEnabled(rule)).length}
                    enabled
                  </small>
                </span>
                <select
                  class="select compact"
                  disabled={isLintConfigLoading || isLintConfigSaving}
                  value={groupState === "mixed" ? "default" : groupState}
                  on:click|stopPropagation={() => {}}
                  on:change={(event) => setGroupOverride(group, event.currentTarget.value as RuleOverride)}
                >
                  <option value="default">{groupState === "mixed" ? "Mixed" : "Default"}</option>
                  <option value="on">Enable all</option>
                  <option value="off">Disable all</option>
                </select>
              </button>

              {#if expanded}
                <div class="rules-list">
                  {#each group.matchedRules as rule}
                    <div class:customized={getRuleValue(rule.id) !== "default"} class="rule-row">
                      <div class="grow">
                        <strong>{rule.name}</strong>
                        {#if getRuleValue(rule.id) !== "default"}
                          <span class="pill amber">Customized</span>
                        {/if}
                        <p>{rule.desc}</p>
                      </div>
                      <select
                        class="select compact"
                        disabled={isLintConfigLoading || isLintConfigSaving}
                        value={getRuleValue(rule.id)}
                        on:change={(event) => setRuleOverride(rule.id, event.currentTarget.value as RuleOverride)}
                      >
                        {#each defaultRuleOptions as option}
                          <option value={option.value}>{option.label}</option>
                        {/each}
                      </select>
                    </div>
                  {/each}
                </div>
              {/if}
            </article>
          {/each}
        </div>
      </section>

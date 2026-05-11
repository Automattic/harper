<script lang="ts">
  import { onMount } from "svelte";
  import type { LintConfig } from "harper.js";
  import { Client } from "$lib/client";
  import { RULE_GROUPS, type RuleGroup, type RuleItem, type RuleOverride } from "../settings-data";

  type MatchedRuleGroup = RuleGroup & { matchedRules: RuleItem[] };

  const defaultRuleOptions: { value: RuleOverride; label: string }[] = [
    { value: "default", label: "Default" },
    { value: "on", label: "Enabled" },
    { value: "off", label: "Disabled" },
  ];

  let lintConfig: LintConfig | null = null;
  let rules: Record<string, RuleOverride> = {};
  let rulesSearch = "";
  let expandedGroups: Record<string, boolean> = {};
  let isLintConfigLoading = true;
  let isLintConfigSaving = false;
  let lintConfigError = "";

  $: rulesQuery = rulesSearch.trim().toLowerCase();
  $: displayedRules = getDisplayedRules();
  $: enabledRuleCount = displayedRules.filter((rule) => isRuleEnabled(rule)).length;
  $: customizedRuleCount = Object.values(rules).filter((value) => value !== "default").length;
  $: filteredRuleGroups = getFilteredRuleGroups(rulesQuery);

  onMount(() => {
    void loadLintConfig();

    const refreshLintConfig = () => {
      if (!isLintConfigSaving) {
        void loadLintConfig();
      }
    };

    window.addEventListener("focus", refreshLintConfig);

    return () => {
      window.removeEventListener("focus", refreshLintConfig);
    };
  });

  async function loadLintConfig() {
    isLintConfigLoading = true;
    lintConfigError = "";

    try {
      const fetchedLintConfig = await Client.getLintConfig();
      lintConfig = fetchedLintConfig;
      rules = rulesFromLintConfig(fetchedLintConfig);
    } catch (error) {
      lintConfigError = `Unable to load lint config: ${error}`;
    } finally {
      isLintConfigLoading = false;
    }
  }

  function rulesFromLintConfig(config: LintConfig): Record<string, RuleOverride> {
    return Object.fromEntries(
      Object.entries(config).map(([ruleId, value]) => [ruleId, lintValueToRuleOverride(value)]),
    );
  }

  function lintValueToRuleOverride(value: boolean | null | undefined): RuleOverride {
    if (value === true) return "on";
    if (value === false) return "off";
    return "default";
  }

  function ruleOverrideToLintValue(value: RuleOverride): boolean | null {
    if (value === "on") return true;
    if (value === "off") return false;
    return null;
  }

  function ruleLabelFromKey(key: string) {
    return key
      .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
      .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
      .trim();
  }

  function getRuleGroups(): RuleGroup[] {
    if (!lintConfig) {
      return RULE_GROUPS;
    }

    const rules = Object.keys(lintConfig)
      .sort((a, b) => ruleLabelFromKey(a).localeCompare(ruleLabelFromKey(b)))
      .map((ruleId) => ({
        id: ruleId,
        name: ruleLabelFromKey(ruleId),
        desc: "Harper rule from the current app configuration.",
      }));

    return [
      {
        id: "harper-rules",
        title: "Harper Rules",
        desc: "Rules loaded from the app's current lint configuration.",
        rules,
      },
    ];
  }

  function getDisplayedRules() {
    return getRuleGroups().flatMap((group) => group.rules);
  }

  async function saveLintConfig(nextLintConfig: LintConfig, nextRules: Record<string, RuleOverride>) {
    const previousLintConfig = lintConfig;
    const previousRules = rules;

    lintConfig = nextLintConfig;
    rules = nextRules;
    isLintConfigSaving = true;
    lintConfigError = "";

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
    config[ruleId] = ruleOverrideToLintValue(value);
  }

  function getFilteredRuleGroups(query: string): MatchedRuleGroup[] {
    const ruleGroups = getRuleGroups();

    if (!query) {
      return ruleGroups.map((group) => ({ ...group, matchedRules: group.rules }));
    }

    return ruleGroups.map((group) => {
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
    }).filter((group): group is MatchedRuleGroup => group !== null);
  }

  function getRuleOptions(rule: RuleItem) {
    return rule.states ?? defaultRuleOptions;
  }

  function getRuleValue(ruleId: string): RuleOverride {
    return rules[ruleId] ?? "default";
  }

  function isRuleEnabled(rule: RuleItem) {
    const value = getRuleValue(rule.id);
    return value !== "off" && value !== "forbid";
  }

  async function setRuleOverride(ruleId: string, value: RuleOverride) {
    const nextRules = { ...rules };

    if (value === "default") {
      delete nextRules[ruleId];
    } else {
      nextRules[ruleId] = value;
    }

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
      if (value === "default") {
        delete nextRules[rule.id];
      } else {
        nextRules[rule.id] = value;
      }

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

  function getGroupState(group: RuleGroup): RuleOverride | "mixed" {
    const values = group.rules.map((rule) => getRuleValue(rule.id));
    const first = values[0];
    return values.every((value) => value === first) ? first : "mixed";
  }

  async function resetRules() {
    if (!lintConfig) {
      rules = {};
      return;
    }

    const nextLintConfig = { ...lintConfig };
    const nextRules: Record<string, RuleOverride> = {};

    for (const rule of displayedRules) {
      nextLintConfig[rule.id] = null;
    }

    await saveLintConfig(nextLintConfig, nextRules);
  }

  async function disableRules() {
    const nextRules = Object.fromEntries(displayedRules.map((rule) => [rule.id, "off" as RuleOverride]));

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
                        {#each getRuleOptions(rule) as option}
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

<script lang="ts">
  import { onMount } from "svelte";
  import type { LintConfig } from "harper.js";
  import { Client } from "$lib/client";
  import SettingsSidebar from "./SettingsSidebar.svelte";
  import {
    APP_PICKER_CANDIDATES,
    BUILTIN_APPS,
    DIALECT_OPTIONS,
    RULE_GROUPS,
    createInitialSettingsState,
    type AppIntegration,
    type RuleGroup,
    type RuleItem,
    type RuleOverride,
    type SectionId,
    type SettingsState,
    type Weirpack,
  } from "./settings-data";

  type SetupStep = {
    id: "accessibility" | "integration" | "test-drive";
    title: string;
    desc: string;
    required: boolean;
    done: boolean;
    locked: boolean;
    actionLabel: string;
    actionVariant: "default" | "primary";
    action: () => void;
  };

  type MatchedRuleGroup = RuleGroup & { matchedRules: RuleItem[] };

  let active: SectionId = "getting-started";
  let state: SettingsState = createInitialSettingsState();
  let contentEl: HTMLElement;

  let newDictionaryWord = "";
  let dictionarySearch = "";
  let rulesSearch = "";
  let expandedGroups: Record<string, boolean> = {};
  let appPickerOpen = false;
  let appSearch = "";
  let packDragState: "idle" | "dragging" = "idle";
  let editingPackId: string | null = null;
  let editingPackName = "";
  let lintConfig: LintConfig | null = null;
  let isLintConfigLoading = true;
  let isLintConfigSaving = false;
  let lintConfigError = "";

  const titleMap: Record<SectionId, string> = {
    "getting-started": "Getting Started",
    general: "General",
    writing: "Writing",
    dictionary: "Dictionary",
    shortcuts: "Shortcuts",
    rules: "Rules",
    weirpacks: "Weirpacks",
    integrations: "Integrations",
    about: "About",
  };

  const shortcutItems = [
    { id: "show-menu", label: "Show Harper menu", keys: ["Shift", "Cmd", "H"] },
    { id: "quick-review", label: "Open quick review", keys: ["Ctrl", "Cmd", "Space"] },
    { id: "accept-last", label: "Apply last suggestion", keys: ["Ctrl", "E"] },
    { id: "dismiss-last", label: "Dismiss last suggestion", keys: ["Ctrl", "D"] },
    { id: "toggle-pause", label: "Pause or resume checking", keys: ["Ctrl", "Shift", "P"] },
    { id: "add-word", label: "Add word to dictionary", keys: ["Ctrl", "Shift", "A"] },
    { id: "next-suggestion", label: "Jump to next suggestion", keys: [] },
  ];

  const defaultRuleOptions: { value: RuleOverride; label: string }[] = [
    { value: "default", label: "Default" },
    { value: "on", label: "Enabled" },
    { value: "off", label: "Disabled" },
  ];

  $: title = titleMap[active];
  $: hasSetupAlert = state.setup.accessibility !== "granted";
  $: setupSteps = buildSetupSteps();
  $: setupCompletedCount = setupSteps.filter((step) => step.done).length;
  $: setupAllDone = setupSteps.every((step) => step.done);
  $: setupRequiredDone = setupSteps.filter((step) => step.required).every((step) => step.done);
  $: filteredWords = state.dictionary.filter((word) =>
    word.toLowerCase().includes(dictionarySearch.trim().toLowerCase()),
  );
  $: rulesQuery = rulesSearch.trim().toLowerCase();
  $: filteredRuleGroups = getFilteredRuleGroups(rulesQuery);
  $: displayedRules = getDisplayedRules();
  $: enabledRuleCount = displayedRules.filter((rule) => isRuleEnabled(rule)).length;
  $: customizedRuleCount = Object.values(state.rules).filter((value) => value !== "default").length;
  $: integrationApps = [
    ...BUILTIN_APPS.filter((app) => !state.removedBuiltins.includes(app.id)),
    ...state.customApps,
  ];
  $: pickerItems = getPickerItems();
  $: enabledPackCount = state.weirpacks.filter((pack) => pack.enabled).length;
  $: enabledPackRules = state.weirpacks
    .filter((pack) => pack.enabled)
    .reduce((total, pack) => total + pack.ruleCount, 0);

  $: if (contentEl && active) {
    contentEl.scrollTop = 0;
  }

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

  function updateState(patch: Partial<SettingsState>) {
    state = { ...state, ...patch };
  }

  function updateSetup(patch: Partial<SettingsState["setup"]>) {
    updateState({ setup: { ...state.setup, ...patch } });
  }

  async function loadLintConfig() {
    isLintConfigLoading = true;
    lintConfigError = "";

    try {
      const fetchedLintConfig = await Client.getLintConfig();
      lintConfig = fetchedLintConfig;
      updateState({ rules: rulesFromLintConfig(fetchedLintConfig) });
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
    const previousRules = state.rules;

    lintConfig = nextLintConfig;
    updateState({ rules: nextRules });
    isLintConfigSaving = true;
    lintConfigError = "";

    try {
      await Client.setLintConfig(nextLintConfig);
    } catch (error) {
      lintConfig = previousLintConfig;
      updateState({ rules: previousRules });
      lintConfigError = `Unable to save lint config: ${error}`;
    } finally {
      isLintConfigSaving = false;
    }
  }

  function setLintConfigRuleValue(config: LintConfig, ruleId: string, value: RuleOverride) {
    config[ruleId] = ruleOverrideToLintValue(value);
  }

  function buildSetupSteps(): SetupStep[] {
    const accessibilityDone = state.setup.accessibility === "granted";
    const integrationDone = state.setup.integration === "selected";
    const testDriveDone = state.setup.testDrive === "completed";

    return [
      {
        id: "accessibility",
        title: "Grant Accessibility permission",
        desc: "Open system settings and grant Harper access to the Accessibility system.",
        required: true,
        done: accessibilityDone,
        locked: false,
        actionLabel: accessibilityDone ? "Granted" : "Open System Settings",
        actionVariant: accessibilityDone ? "default" : "primary",
        action: () => updateSetup({ accessibility: "granted" }),
      },
      {
        id: "integration",
        title: "Pick an app to test",
        desc: "Start with TextEdit, then add more apps from Integrations when you are ready.",
        required: true,
        done: integrationDone,
        locked: !accessibilityDone,
        actionLabel: integrationDone ? "Manage" : "Browse apps",
        actionVariant: "default",
        action: () => (active = "integrations"),
      },
      {
        id: "test-drive",
        title: "Take a test drive",
        desc: 'Open TextEdit, type "its not alot of fun", and watch Harper underline the mistakes.',
        required: false,
        done: testDriveDone,
        locked: !accessibilityDone || !integrationDone,
        actionLabel: testDriveDone ? "Run again" : "Launch TextEdit",
        actionVariant: testDriveDone ? "default" : "primary",
        action: () => updateSetup({ testDrive: "completed" }),
      },
    ];
  }

  function setIntegration(id: string, enabled: boolean) {
    updateState({ integrations: { ...state.integrations, [id]: enabled } });
  }

  function enableTextEditForSetup() {
    setIntegration("textedit", true);
    updateSetup({ integration: "selected" });
  }

  function addDictionaryWord() {
    const word = newDictionaryWord.trim();

    if (!word || state.dictionary.includes(word)) {
      newDictionaryWord = "";
      return;
    }

    updateState({ dictionary: [word, ...state.dictionary] });
    newDictionaryWord = "";
  }

  function removeDictionaryWord(word: string) {
    updateState({ dictionary: state.dictionary.filter((item) => item !== word) });
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
    return state.rules[ruleId] ?? "default";
  }

  function isRuleEnabled(rule: RuleItem) {
    const value = getRuleValue(rule.id);
    return value !== "off" && value !== "forbid";
  }

  async function setRuleOverride(ruleId: string, value: RuleOverride) {
    const rules = { ...state.rules };

    if (value === "default") {
      delete rules[ruleId];
    } else {
      rules[ruleId] = value;
    }

    if (!lintConfig) {
      updateState({ rules });
      return;
    }

    const nextLintConfig = { ...lintConfig };
    setLintConfigRuleValue(nextLintConfig, ruleId, value);
    await saveLintConfig(nextLintConfig, rules);
  }

  async function setGroupOverride(group: RuleGroup, value: RuleOverride) {
    const rules = { ...state.rules };
    const nextLintConfig = lintConfig ? { ...lintConfig } : null;

    for (const rule of group.rules) {
      if (value === "default") {
        delete rules[rule.id];
      } else {
        rules[rule.id] = value;
      }

      if (nextLintConfig) {
        setLintConfigRuleValue(nextLintConfig, rule.id, value);
      }
    }

    if (!nextLintConfig) {
      updateState({ rules });
      return;
    }

    await saveLintConfig(nextLintConfig, rules);
  }

  function getGroupState(group: RuleGroup): RuleOverride | "mixed" {
    const values = group.rules.map((rule) => getRuleValue(rule.id));
    const first = values[0];
    return values.every((value) => value === first) ? first : "mixed";
  }

  async function resetRules() {
    if (!lintConfig) {
      updateState({ rules: {} });
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
    const rules = Object.fromEntries(displayedRules.map((rule) => [rule.id, "off" as RuleOverride]));

    if (!lintConfig) {
      updateState({ rules });
      return;
    }

    const nextLintConfig = { ...lintConfig };

    for (const rule of displayedRules) {
      nextLintConfig[rule.id] = false;
    }

    await saveLintConfig(nextLintConfig, rules);
  }

  function toggleGroup(groupId: string) {
    if (rulesQuery) return;
    expandedGroups = { ...expandedGroups, [groupId]: !expandedGroups[groupId] };
  }

  function getPickerItems() {
    const removedBuiltins = BUILTIN_APPS.filter((app) => state.removedBuiltins.includes(app.id));
    const visibleIds = new Set(integrationApps.map((app) => app.id));
    const query = appSearch.trim().toLowerCase();

    return [...removedBuiltins, ...APP_PICKER_CANDIDATES].filter(
      (app) =>
        !visibleIds.has(app.id) &&
        (!query || app.name.toLowerCase().includes(query) || app.kind.toLowerCase().includes(query)),
    );
  }

  function addApp(app: AppIntegration) {
    const isBuiltin = BUILTIN_APPS.some((candidate) => candidate.id === app.id);

    if (isBuiltin) {
      updateState({
        removedBuiltins: state.removedBuiltins.filter((id) => id !== app.id),
        integrations: { ...state.integrations, [app.id]: true },
      });
    } else if (!state.customApps.some((candidate) => candidate.id === app.id)) {
      updateState({
        customApps: [...state.customApps, { ...app, custom: true }],
        integrations: { ...state.integrations, [app.id]: true },
      });
    }

    appPickerOpen = false;
    appSearch = "";
  }

  function removeApp(app: AppIntegration) {
    const integrations = { ...state.integrations };
    delete integrations[app.id];

    updateState({
      customApps: state.customApps.filter((candidate) => candidate.id !== app.id),
      removedBuiltins: app.custom ? state.removedBuiltins : [...state.removedBuiltins, app.id],
      integrations,
    });
  }

  function addFiles(files: FileList | null) {
    if (!files) return;

    for (const file of Array.from(files)) {
      const id = `pack-${Date.now()}-${Math.round(Math.random() * 1000)}`;
      const baseName = file.name
        .replace(/\.(weirpack|json|wpck)$/i, "")
        .replace(/[-_]/g, " ")
        .replace(/\b\w/g, (letter) => letter.toUpperCase());
      const pack: Weirpack = {
        id,
        name: baseName || "Custom Pack",
        filename: file.name || "pack.weirpack",
        size: file.size || 4096,
        enabled: true,
        ruleCount: Math.max(8, Math.round(file.size / 800)),
        addedAt: new Date().toISOString(),
        installState: "installing",
      };

      updateState({ weirpacks: [...state.weirpacks, pack] });

      window.setTimeout(() => {
        updatePack(id, { installState: "installed" });
      }, 500);
    }
  }

  function updatePack(id: string, patch: Partial<Weirpack>) {
    updateState({
      weirpacks: state.weirpacks.map((pack) =>
        pack.id === id ? { ...pack, ...patch } : pack,
      ),
    });
  }

  function removePack(id: string) {
    updateState({ weirpacks: state.weirpacks.filter((pack) => pack.id !== id) });
  }

  function startRenamePack(pack: Weirpack) {
    editingPackId = pack.id;
    editingPackName = pack.name;
  }

  function commitRenamePack() {
    if (!editingPackId) return;
    const name = editingPackName.trim();

    if (name) {
      updatePack(editingPackId, { name });
    }

    editingPackId = null;
    editingPackName = "";
  }

  function formatSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
  }

  function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }
</script>

<div class="settings-shell">
  <SettingsSidebar bind:active {hasSetupAlert} />

  <main bind:this={contentEl} class="content" aria-label={title}>
    {#if active === "getting-started"}
      <section>
        {#if setupAllDone}
          <div class="success-banner">
            <div class="big-mark green">
              <span class="settings-icon icon-check" aria-hidden="true"></span>
            </div>
            <div class="grow">
              <h2>You're all set</h2>
              <p>
                Harper is ready to check writing in the apps you choose. You can revisit any section
                from the sidebar.
              </p>
            </div>
            <button class="button" type="button" on:click={() => updateSetup({ testDrive: "not_started" })}>
              Walk through again
            </button>
          </div>
        {:else}
          {#if state.setup.accessibility !== "granted"}
            <div class="warning-banner">
              <div class="big-mark amber">!</div>
              <div>
                <strong>Harper is not checking anything yet</strong>
                <p>Grant Accessibility permission so Harper can find text and surface suggestions.</p>
              </div>
            </div>
          {/if}

          <div class="hero-copy">
            <div class="eyebrow">Getting started</div>
            <h1>Let's get Harper up and running.</h1>
            <div class="progress-row">
              <div class="progress-track">
                <div class="progress-fill" style={`width: ${(setupCompletedCount / setupSteps.length) * 100}%`}></div>
              </div>
              <span>{setupCompletedCount} of {setupSteps.length}</span>
            </div>
          </div>
        {/if}

        <div class="step-list">
          {#each setupSteps as step, index}
            <div class:done={step.done} class:locked={step.locked} class="step-row">
              <div class="step-dot">
                {#if step.done}
                  <span class="settings-icon icon-check" aria-hidden="true"></span>
                {:else}
                  {index + 1}
                {/if}
              </div>
              <div class="grow">
                <div class="step-heading">
                  <strong>{step.title}</strong>
                  {#if !step.required && !step.done}
                    <span class="pill">Optional</span>
                  {/if}
                </div>
                <p>{step.desc}</p>

                {#if step.id === "integration" && state.setup.accessibility === "granted" && state.setup.integration !== "selected"}
                  <div class="detected-app">
                    <div class="app-tile" style="--app-tint: #5a5f68">T</div>
                    <div class="grow">
                      <strong>TextEdit detected</strong>
                      <p>A good starter app for trying Harper.</p>
                    </div>
                    <button class="button primary" type="button" on:click={enableTextEditForSetup}>
                      Enable
                    </button>
                  </div>
                {/if}
              </div>
              <button
                class={`button ${step.actionVariant === "primary" ? "primary" : ""}`}
                type="button"
                disabled={step.locked}
                on:click={step.action}
              >
                {step.actionLabel}
              </button>
            </div>
          {/each}
        </div>

        <div class="note-strip">
          <strong>On-device by default.</strong>
          <span>Your writing stays on this Mac in this demo surface.</span>
        </div>
      </section>
    {:else if active === "general"}
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
                class:checked={state.menuBar}
                class="checkbox"
                type="button"
                role="checkbox"
                aria-checked={state.menuBar}
                on:click={() => updateState({ menuBar: !state.menuBar })}
              >
                {#if state.menuBar}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>

            <div class="inline-row">
              <label for="menu-bar-click">Click the icon to:</label>
              <select
                id="menu-bar-click"
                class="select"
                bind:value={state.menuBarClick}
                on:change={() => updateState({ menuBarClick: state.menuBarClick })}
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
                class:checked={state.launchAtStartup}
                class="checkbox"
                type="button"
                role="checkbox"
                aria-checked={state.launchAtStartup}
                on:click={() => updateState({ launchAtStartup: !state.launchAtStartup })}
              >
                {#if state.launchAtStartup}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>

            <div class="row top">
              <div>
                <strong>Automatically check for updates</strong>
                <p>Harper will check for new versions weekly.</p>
              </div>
              <button
                class:checked={state.autoUpdate}
                class="checkbox"
                type="button"
                role="checkbox"
                aria-checked={state.autoUpdate}
                on:click={() => updateState({ autoUpdate: !state.autoUpdate })}
              >
                {#if state.autoUpdate}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
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
              bind:value={state.dialect}
              on:change={() => updateState({ dialect: state.dialect })}
            >
              {#each DIALECT_OPTIONS as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Updates</div>
          <div class="row top">
            <div>
              <strong>You're up to date</strong>
              <p>Harper 1.4.2, released April 18, 2026.</p>
            </div>
            <button class="button" type="button">Check now</button>
          </div>
        </div>
      </section>
    {:else if active === "writing"}
      <section>
        <div class="stanza">
          <div class="eyebrow">Grammar checking</div>
          <p class="section-copy">
            Tune how assertive Harper is. Lower strictness surfaces fewer suggestions.
          </p>
          <div class="rows">
            <div class="inline-row">
              <label for="strictness">Strictness:</label>
              <select
                id="strictness"
                class="select"
                bind:value={state.strictness}
                on:change={() => updateState({ strictness: state.strictness })}
              >
                <option value="relaxed">Relaxed</option>
                <option value="standard">Standard</option>
                <option value="strict">Strict</option>
              </select>
            </div>

            <div class="row">
              <div>
                <strong>Check while I type</strong>
                <p>Suggestions appear as you write.</p>
              </div>
              <button
                class:checked={state.liveCheck}
                class="checkbox"
                type="button"
                role="checkbox"
                aria-checked={state.liveCheck}
                on:click={() => updateState({ liveCheck: !state.liveCheck })}
              >
                {#if state.liveCheck}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>

            <div class="row top">
              <div>
                <strong>Respect code blocks and quotes</strong>
                <p>Harper skips fenced code, inline code, and block quotes when checking Markdown.</p>
              </div>
              <button
                class:checked={state.respectCode}
                class="checkbox"
                type="button"
                role="checkbox"
                aria-checked={state.respectCode}
                on:click={() => updateState({ respectCode: !state.respectCode })}
              >
                {#if state.respectCode}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>
          </div>
        </div>
      </section>
    {:else if active === "dictionary"}
      <section>
        <div class="stanza">
          <div class="eyebrow">User Dictionary</div>
          <p class="section-copy">
            Words and names Harper should never flag. This list is local demo state.
          </p>

          <div class="add-row">
            <input
              class="text-field"
              type="text"
              placeholder="Add a word..."
              bind:value={newDictionaryWord}
              on:keydown={(event) => event.key === "Enter" && addDictionaryWord()}
            />
            <button class="button primary" type="button" on:click={addDictionaryWord}>Add</button>
          </div>

          <div class="list-card">
            <div class="search-strip">
              <span class="settings-icon icon-search" aria-hidden="true"></span>
              <input
                type="text"
                placeholder={`Search ${state.dictionary.length} words`}
                bind:value={dictionarySearch}
              />
              <span>{filteredWords.length} of {state.dictionary.length}</span>
            </div>

            <div class="dictionary-list">
              {#if filteredWords.length === 0}
                <div class="empty">No matching words.</div>
              {:else}
                {#each filteredWords as word}
                  <div class="list-row">
                    <code>{word}</code>
                    <button
                      class="icon-button danger"
                      type="button"
                      aria-label={`Remove ${word}`}
                      on:click={() => removeDictionaryWord(word)}
                    >
                      <span class="settings-icon icon-x" aria-hidden="true"></span>
                    </button>
                  </div>
                {/each}
              {/if}
            </div>
          </div>

          <div class="actions-row">
            <button class="button" type="button">Import from file...</button>
            <button class="button" type="button">Export dictionary</button>
            <span class="spacer"></span>
            <button class="button danger" type="button" on:click={() => updateState({ dictionary: [] })}>
              Clear all
            </button>
          </div>
        </div>
      </section>
    {:else if active === "shortcuts"}
      <section>
        <div class="stanza">
          <div class="eyebrow">Keyboard Shortcuts</div>
          <p class="section-copy">
            Global shortcuts work from anywhere on macOS. Click a shortcut to preview recording.
          </p>

          <div class="shortcut-list">
            {#each shortcutItems as item}
              <button class="shortcut-row" type="button">
                <span>{item.label}</span>
                {#if item.keys.length > 0}
                  <span class="kbd-group">
                    {#each item.keys as key}
                      <kbd>{key}</kbd>
                    {/each}
                  </span>
                {:else}
                  <em>Click to record...</em>
                {/if}
              </button>
            {/each}
          </div>

          <div class="row top">
            <div>
              <strong>Allow shortcuts while other apps are focused</strong>
              <p>When off, Harper shortcuts only work while the Harper window is active.</p>
            </div>
            <button
              class:checked={state.globalShortcuts}
              class="checkbox"
              type="button"
              role="checkbox"
              aria-checked={state.globalShortcuts}
              on:click={() => updateState({ globalShortcuts: !state.globalShortcuts })}
            >
              {#if state.globalShortcuts}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
            </button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Activation</div>
          <div class="row top">
            <div>
              <strong>Activation key</strong>
              <p>Require a modifier key to enable Harper checking in a window.</p>
            </div>
            <select
              class="select"
              bind:value={state.activationKey}
              on:change={() => updateState({ activationKey: state.activationKey })}
            >
              <option value="off">Off</option>
              <option value="option">Option</option>
              <option value="control">Control</option>
              <option value="shift">Shift</option>
            </select>
          </div>
        </div>
      </section>
    {:else if active === "rules"}
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
                <span class:expanded class="chevron">v</span>
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
    {:else if active === "weirpacks"}
      <section>
        <div class="stanza">
          <div class="eyebrow">Weirpacks</div>
          <p class="section-copy">
            Bundles of custom rules that can be layered on top of Harper's built-in checks.
          </p>

          <label
            class:dragging={packDragState === "dragging"}
            class="drop-zone"
            on:dragover={(event) => {
              event.preventDefault();
              packDragState = "dragging";
            }}
            on:dragleave={() => (packDragState = "idle")}
            on:drop={(event) => {
              event.preventDefault();
              packDragState = "idle";
              addFiles(event.dataTransfer?.files ?? null);
            }}
          >
            <span class="big-mark purple">
              <span class="settings-icon icon-upload" aria-hidden="true"></span>
            </span>
            <span class="grow">
              <strong>{packDragState === "dragging" ? "Drop to install" : "Install a Weirpack"}</strong>
              <p>Drag a .weirpack file here, or click to browse. Multiple files are OK.</p>
            </span>
            <span class="button primary">Choose file...</span>
            <input
              type="file"
              accept=".weirpack,.json,.wpck"
              multiple
              on:change={(event) => {
                addFiles(event.currentTarget.files);
                event.currentTarget.value = "";
              }}
            />
          </label>

          <p class="muted">
            {state.weirpacks.length} packs installed, {enabledPackCount} active, {enabledPackRules}
            extra rules.
          </p>
        </div>

        <div class="stanza">
          <div class="eyebrow">Installed packs</div>
          <div class="list-card">
            {#each state.weirpacks as pack}
              <div class:disabled={!pack.enabled} class="pack-row">
                <div class="pack-icon">
                  <span class="settings-icon icon-package" aria-hidden="true"></span>
                </div>
                <div class="grow">
                  {#if editingPackId === pack.id}
                    <input
                      class="inline-edit"
                      type="text"
                      bind:value={editingPackName}
                      on:blur={commitRenamePack}
                      on:keydown={(event) => {
                        if (event.key === "Enter") commitRenamePack();
                        if (event.key === "Escape") {
                          editingPackId = null;
                          editingPackName = "";
                        }
                      }}
                    />
                  {:else}
                    <button class="pack-title" type="button" on:dblclick={() => startRenamePack(pack)}>
                      {pack.name}
                    </button>
                  {/if}
                  <p>
                    <code>{pack.filename}</code>
                    <span> - {pack.ruleCount} rules - {formatSize(pack.size)} - added {formatDate(pack.addedAt)}</span>
                  </p>
                  {#if pack.installState !== "installed"}
                    <span class={`status ${pack.installState}`}>{pack.installState}</span>
                  {/if}
                </div>
                <button
                  class="icon-button danger"
                  type="button"
                  aria-label={`Remove ${pack.name}`}
                  on:click={() => removePack(pack.id)}
                >
                  <span class="settings-icon icon-trash" aria-hidden="true"></span>
                </button>
                <button
                  class:checked={pack.enabled}
                  class="toggle"
                  type="button"
                  role="switch"
                  aria-checked={pack.enabled}
                  aria-label={`Toggle ${pack.name}`}
                  on:click={() => updatePack(pack.id, { enabled: !pack.enabled })}
                >
                  <span></span>
                </button>
              </div>
            {/each}
          </div>
        </div>
      </section>
    {:else if active === "integrations"}
      <section>
        <div class="stanza">
          <div class="eyebrow">App integrations</div>
          <div class="row top">
            <div>
              <strong>Watch everywhere</strong>
              <p>Harper checks writing in any app that supports text input.</p>
            </div>
            <button
              class:checked={state.watchEverywhere}
              class="toggle"
              type="button"
              role="switch"
              aria-checked={state.watchEverywhere}
              aria-label="Toggle watch everywhere"
              on:click={() => updateState({ watchEverywhere: !state.watchEverywhere })}
            >
              <span></span>
            </button>
          </div>
        </div>

        {#if !state.watchEverywhere}
          <div class="divider"></div>

          <div class="stanza">
            <div class="eyebrow">Selected apps</div>
            <p class="section-copy">Harper will only watch the apps you enable here.</p>

            <div class="list-card">
              {#each integrationApps as app}
                <div class="app-row">
                  <div class="app-tile" style={`--app-tint: ${app.tint}`}>{app.name[0]}</div>
                  <div class="grow">
                    <strong>{app.name}</strong>
                    <span class="pill">{app.kind}</span>
                    {#if app.custom}<span class="pill purple-text">Added</span>{/if}
                    {#if app.note}<p>{app.note}</p>{/if}
                  </div>
                  <button
                    class="icon-button danger"
                    type="button"
                    aria-label={`Remove ${app.name}`}
                    on:click={() => removeApp(app)}
                  >
                    <span class="settings-icon icon-trash" aria-hidden="true"></span>
                  </button>
                  <button
                    class:checked={state.integrations[app.id] ?? false}
                    class="toggle"
                    type="button"
                    role="switch"
                    aria-checked={state.integrations[app.id] ?? false}
                    aria-label={`Toggle ${app.name}`}
                    on:click={() => setIntegration(app.id, !(state.integrations[app.id] ?? false))}
                  >
                    <span></span>
                  </button>
                </div>
              {/each}
            </div>

            <div class="actions-row">
              <button class="button" type="button" on:click={() => (appPickerOpen = true)}>
                Add application...
              </button>
              <span class="muted">Choose any app from your Applications folder.</span>
            </div>
          </div>

          <div class="divider"></div>

          <div class="stanza">
            <div class="eyebrow">New apps</div>
            <div class="row top">
              <div>
                <strong>Enable new apps automatically</strong>
                <p>When you launch a supported app for the first time, turn integration on by default.</p>
              </div>
              <button
                class:checked={state.autoIntegrate}
                class="checkbox"
                type="button"
                role="checkbox"
                aria-checked={state.autoIntegrate}
                on:click={() => updateState({ autoIntegrate: !state.autoIntegrate })}
              >
                {#if state.autoIntegrate}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>
          </div>
        {/if}
      </section>
    {:else if active === "about"}
      <section class="about">
        <div class="about-mark">H</div>
        <h1>Harper for Mac</h1>
        <p class="muted">Version 1.4.2 (build 2048)</p>
        <p>
          An open-source grammar checker that runs entirely on your device. No accounts, no
          telemetry, no cloud.
        </p>
        <div class="actions-row center">
          <button class="button" type="button">Release notes</button>
          <button class="button" type="button">Source on GitHub</button>
          <button class="button" type="button">Report an issue</button>
        </div>
        <div class="about-footer">
          Harper is free software released under the Apache 2.0 license.
          <br />
          Copyright 2026 The Harper Contributors.
        </div>
      </section>
    {/if}
  </main>

  {#if appPickerOpen}
    <div
      class="modal-backdrop"
      role="button"
      tabindex="0"
      aria-label="Close application picker"
      on:click={() => (appPickerOpen = false)}
      on:keydown={(event) => {
        if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
          appPickerOpen = false;
        }
      }}
    >
      <div
        class="modal"
        role="dialog"
        tabindex="-1"
        aria-label="Choose an application"
        on:click|stopPropagation={() => {}}
        on:keydown|stopPropagation={(event) => {
          if (event.key === "Escape") {
            appPickerOpen = false;
          }
        }}
      >
        <div class="modal-head">
          <strong>Choose an application</strong>
          <span>Applications - /Applications</span>
        </div>
        <div class="modal-search">
          <span class="settings-icon icon-search" aria-hidden="true"></span>
          <input type="text" placeholder="Search Applications..." bind:value={appSearch} />
        </div>
        <div class="modal-list">
          {#if pickerItems.length === 0}
            <div class="empty">No applications match "{appSearch}".</div>
          {:else}
            {#each pickerItems as app}
              <button class="picker-row" type="button" on:click={() => addApp(app)}>
                <span class="app-tile" style={`--app-tint: ${app.tint}`}>{app.name[0]}</span>
                <span class="grow">
                  <strong>{app.name}</strong>
                  <small>Application</small>
                </span>
                <span>{app.kind}</span>
              </button>
            {/each}
          {/if}
        </div>
        <div class="modal-actions">
          <button class="button" type="button" on:click={() => (appPickerOpen = false)}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(body.settings-view) {
    background: #fbfaf7;
  }

  :global(*) {
    box-sizing: border-box;
  }

  .settings-shell {
    --settings-amber: #b06a1b;
    --settings-amber-soft: #d98b3a;
    --settings-bg-window: #fbfaf7;
    --settings-bg-sidebar: #f1ede5;
    --settings-ink: #1c1a16;
    --settings-ink-2: #4a463e;
    --settings-ink-3: #807a6e;
    --settings-ink-4: #ada79a;
    --settings-line: rgba(28, 26, 22, 0.09);
    --settings-line-strong: rgba(28, 26, 22, 0.14);
    --settings-accent: #2a6bd8;
    height: 100vh;
    width: 100vw;
    display: flex;
    overflow: hidden;
    background: var(--settings-bg-window);
    color: var(--settings-ink);
    font-family:
      Inter,
      -apple-system,
      BlinkMacSystemFont,
      "SF Pro Text",
      "Helvetica Neue",
      sans-serif;
    font-size: 13px;
    -webkit-font-smoothing: antialiased;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 36px 44px;
  }

  section {
    max-width: 720px;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    font-size: 24px;
    line-height: 1.2;
    font-weight: 700;
  }

  h2 {
    font-size: 16px;
    line-height: 1.3;
    font-weight: 700;
  }

  p {
    color: var(--settings-ink-3);
    line-height: 1.45;
  }

  button,
  input,
  select {
    font: inherit;
  }

  button {
    cursor: default;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .spacer {
    flex: 1;
  }

  .eyebrow {
    margin-bottom: 18px;
    color: var(--settings-ink-3);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .hero-copy {
    margin-bottom: 26px;
  }

  .hero-copy .eyebrow {
    margin-bottom: 8px;
    color: var(--settings-amber);
    letter-spacing: 0.12em;
  }

  .progress-row,
  .actions-row,
  .step-heading {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .progress-row {
    margin-top: 18px;
    color: var(--settings-ink-3);
    font-size: 12px;
  }

  .progress-track {
    width: 260px;
    height: 6px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.08);
  }

  .progress-fill {
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, #d98b3a 0%, #b06a1b 100%);
    transition: width 160ms ease;
  }

  .warning-banner,
  .success-banner,
  .note-strip {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    border-radius: 10px;
  }

  .warning-banner {
    margin-bottom: 22px;
    padding: 14px 18px;
    border: 0.5px solid rgba(201, 70, 20, 0.3);
    background: linear-gradient(180deg, #fff2ed 0%, #ffe4d6 100%);
  }

  .success-banner {
    align-items: center;
    margin-bottom: 26px;
    padding: 24px 28px;
    border: 0.5px solid rgba(30, 120, 70, 0.22);
    background: linear-gradient(150deg, #e7f5ec 0%, #d5efdf 100%);
  }

  .big-mark {
    width: 34px;
    height: 34px;
    flex: 0 0 34px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: #fff;
    font-weight: 700;
  }

  .big-mark.amber {
    background: linear-gradient(180deg, #f07a3a 0%, #c94614 100%);
  }

  .big-mark.green {
    background: linear-gradient(180deg, #3dbd6d 0%, #1f8f5c 100%);
  }

  .big-mark.purple {
    background: linear-gradient(180deg, #b78cf6 0%, #7c3aed 100%);
  }

  .big-mark .settings-icon {
    width: 20px;
    height: 20px;
    flex-basis: 20px;
  }

  .success-banner .big-mark .settings-icon {
    width: 22px;
    height: 22px;
    flex-basis: 22px;
  }

  .step-list,
  .list-card,
  .rule-group {
    overflow: hidden;
    border: 0.5px solid var(--settings-line-strong);
    border-radius: 10px;
    background: #fff;
  }

  .step-row {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 18px 20px;
    border-bottom: 0.5px solid var(--settings-line);
  }

  .step-row:last-child {
    border-bottom: 0;
  }

  .step-row.done {
    background: linear-gradient(180deg, rgba(30, 143, 92, 0.04) 0%, transparent 100%);
  }

  .step-row.locked {
    opacity: 0.52;
  }

  .step-dot {
    width: 26px;
    height: 26px;
    flex: 0 0 26px;
    display: grid;
    place-items: center;
    border: 1px solid rgba(0, 0, 0, 0.2);
    border-radius: 999px;
    color: var(--settings-ink-3);
    font-weight: 700;
  }

  .done .step-dot {
    border: 0;
    background: linear-gradient(180deg, #3dbd6d 0%, #1f8f5c 100%);
    color: #fff;
  }

  .step-dot .settings-icon,
  .search-strip .settings-icon,
  .rule-search .settings-icon,
  .icon-button .settings-icon {
    width: 13px;
    height: 13px;
    flex-basis: 13px;
  }

  .detected-app,
  .note-strip {
    margin-top: 12px;
    padding: 12px 14px;
    border-radius: 8px;
    background: #faf8f3;
  }

  .detected-app {
    display: flex;
    align-items: center;
    gap: 12px;
    border: 0.5px solid var(--settings-line-strong);
  }

  .note-strip {
    margin-top: 22px;
    gap: 8px;
    color: var(--settings-ink-3);
  }

  .stanza {
    margin-bottom: 30px;
  }

  .rows {
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  .row,
  .inline-row {
    display: flex;
    align-items: center;
    gap: 24px;
  }

  .row.top {
    align-items: flex-start;
  }

  .row > div:first-child {
    flex: 1;
    min-width: 0;
  }

  .row strong,
  .inline-row label {
    display: inline-block;
    color: var(--settings-ink);
    font-weight: 600;
    line-height: 1.35;
  }

  .row p,
  .section-copy {
    max-width: 520px;
    margin-top: 3px;
    font-size: 12.5px;
  }

  .inline-row label {
    min-width: 140px;
  }

  .divider {
    height: 1px;
    margin: 20px 0;
    background: var(--settings-line);
  }

  .button,
  .select,
  .text-field {
    height: 28px;
    border-radius: 6px;
    border: 0.5px solid rgba(0, 0, 0, 0.18);
    background: linear-gradient(180deg, #fff 0%, #f4f1ea 100%);
    color: var(--settings-ink);
    box-shadow:
      0 0.5px 0 rgba(0, 0, 0, 0.04),
      inset 0 0.5px 0 rgba(255, 255, 255, 0.8);
  }

  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 12px;
    white-space: nowrap;
    font-weight: 600;
  }

  .button.primary {
    border-color: transparent;
    background: linear-gradient(180deg, var(--settings-amber-soft) 0%, var(--settings-amber) 100%);
    color: #fff;
  }

  .button.danger {
    color: #b42318;
    background: #fff;
    border-color: rgba(180, 35, 24, 0.35);
  }

  .button:disabled {
    opacity: 0.7;
  }

  .select {
    min-width: 145px;
    padding: 0 28px 0 10px;
  }

  .select.wide {
    min-width: 220px;
  }

  .select.compact {
    min-width: 110px;
    height: 26px;
    font-size: 12.5px;
  }

  .text-field {
    background: #fff;
    box-shadow: inset 0 0.5px 1px rgba(0, 0, 0, 0.05);
    padding: 0 10px;
  }

  .checkbox {
    width: 16px;
    height: 16px;
    flex: 0 0 16px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.25);
    background: #fff;
    color: #fff;
    padding: 0;
  }

  .checkbox.checked {
    border-color: rgba(0, 0, 0, 0.15);
    background: var(--settings-accent);
  }

  .checkbox .settings-icon {
    width: 11px;
    height: 11px;
    flex-basis: 11px;
  }

  .toggle {
    width: 34px;
    height: 20px;
    flex: 0 0 34px;
    border: 0;
    border-radius: 999px;
    background: #d4cfc4;
    padding: 2px;
    box-shadow: inset 0 0.5px 1px rgba(0, 0, 0, 0.1);
  }

  .toggle span {
    display: block;
    width: 16px;
    height: 16px;
    border-radius: 999px;
    background: #fff;
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.2),
      0 0.5px 0 rgba(0, 0, 0, 0.1);
    transition: transform 150ms ease;
  }

  .toggle.checked {
    background: var(--settings-accent);
  }

  .toggle.checked span {
    transform: translateX(14px);
  }

  .add-row {
    display: flex;
    gap: 8px;
  }

  .add-row .text-field {
    flex: 1;
  }

  .search-strip,
  .modal-search {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #faf8f3;
  }

  .search-strip {
    padding: 8px 12px;
    border-bottom: 0.5px solid var(--settings-line);
  }

  .search-strip input,
  .modal-search input,
  .rule-search input {
    flex: 1;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--settings-ink);
  }

  .dictionary-list {
    max-height: 320px;
    overflow: auto;
  }

  .list-row,
  .app-row,
  .pack-row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 14px;
    border-bottom: 0.5px solid var(--settings-line);
  }

  .list-row:last-child,
  .app-row:last-child,
  .pack-row:last-child {
    border-bottom: 0;
  }

  code,
  kbd {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  }

  code {
    font-size: 12px;
  }

  .icon-button {
    width: 26px;
    height: 26px;
    flex: 0 0 26px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--settings-ink-3);
  }

  .icon-button:hover {
    background: rgba(0, 0, 0, 0.06);
    color: var(--settings-ink);
  }

  .icon-button.danger:hover {
    background: rgba(180, 35, 24, 0.08);
    color: #b42318;
  }

  .list-row .icon-button .settings-icon,
  .rule-search .icon-button .settings-icon {
    width: 10px;
    height: 10px;
    flex-basis: 10px;
  }

  .empty {
    padding: 34px;
    text-align: center;
    color: var(--settings-ink-3);
  }

  .actions-row {
    margin-top: 14px;
  }

  .actions-row.center {
    justify-content: center;
  }

  .shortcut-list {
    display: flex;
    flex-direction: column;
    margin-bottom: 20px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border: 0;
    border-bottom: 0.5px solid var(--settings-line);
    background: transparent;
    padding: 10px 4px;
    text-align: left;
    color: var(--settings-ink);
  }

  .shortcut-row:hover {
    background: rgba(0, 0, 0, 0.025);
  }

  .kbd-group {
    display: flex;
    gap: 4px;
  }

  kbd {
    min-width: 22px;
    height: 22px;
    display: inline-grid;
    place-items: center;
    border: 0.5px solid rgba(0, 0, 0, 0.18);
    border-radius: 5px;
    background: #fff;
    color: var(--settings-ink);
    padding: 0 6px;
    font-size: 11px;
  }

  em,
  .muted {
    color: var(--settings-ink-3);
    font-size: 12.5px;
  }

  .rules-heading {
    margin-bottom: 18px;
  }

  .rules-heading .eyebrow {
    margin-bottom: 10px;
  }

  .rules-heading p {
    margin-top: 6px;
  }

  .sticky-tools {
    position: sticky;
    top: -36px;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 -44px 16px;
    padding: 10px 44px;
    border-bottom: 0.5px solid var(--settings-line);
    background: rgba(251, 250, 247, 0.88);
    backdrop-filter: blur(14px);
  }

  .rule-search {
    min-width: 260px;
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 9px;
    border: 0.5px solid rgba(0, 0, 0, 0.18);
    border-radius: 7px;
    background: #fff;
  }

  .result-summary {
    margin-bottom: 12px;
    font-size: 12.5px;
  }

  .rule-groups {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .group-head {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    border: 0;
    background: transparent;
    padding: 14px 16px;
    text-align: left;
    color: inherit;
  }

  .group-head:hover {
    background: #faf8f3;
  }

  .group-head p {
    margin-top: 3px;
    font-size: 12.5px;
  }

  .group-head small {
    display: inline-block;
    margin-top: 6px;
    color: var(--settings-ink-4);
    font-size: 11.5px;
  }

  .chevron {
    width: 20px;
    color: var(--settings-ink-3);
    transform: rotate(-90deg);
    transition: transform 140ms ease;
  }

  .chevron.expanded {
    transform: rotate(0deg);
  }

  .rules-list {
    border-top: 0.5px solid var(--settings-line);
  }

  .rule-row {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 11px 16px 11px 48px;
    border-bottom: 0.5px solid var(--settings-line);
  }

  .rule-row:last-child {
    border-bottom: 0;
  }

  .rule-row.customized {
    background: rgba(217, 139, 58, 0.04);
  }

  .rule-row p {
    margin-top: 3px;
    font-size: 12.5px;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    margin-left: 6px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.05);
    color: var(--settings-ink-3);
    padding: 2px 6px;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .pill.amber {
    background: rgba(217, 139, 58, 0.12);
    color: var(--settings-amber);
  }

  .pill.purple-text,
  .purple-text {
    color: #7c3aed;
  }

  .drop-zone {
    display: flex;
    align-items: center;
    gap: 16px;
    border: 1.5px dashed rgba(0, 0, 0, 0.18);
    border-radius: 10px;
    background: #fff;
    padding: 22px 24px;
  }

  .drop-zone.dragging {
    border-color: var(--settings-amber);
    background: rgba(217, 139, 58, 0.06);
  }

  .drop-zone input {
    display: none;
  }

  .pack-row.disabled {
    opacity: 0.62;
  }

  .pack-icon,
  .app-tile {
    display: grid;
    place-items: center;
    color: #fff;
    font-weight: 700;
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.1),
      inset 0 0.5px 0 rgba(255, 255, 255, 0.3);
  }

  .pack-icon {
    width: 32px;
    height: 32px;
    flex: 0 0 32px;
    border-radius: 8px;
    background: linear-gradient(180deg, #b78cf6 0%, #7c3aed 100%);
  }

  .pack-icon .settings-icon {
    width: 16px;
    height: 16px;
    flex-basis: 16px;
  }

  .app-tile {
    width: 28px;
    height: 28px;
    flex: 0 0 28px;
    border-radius: 7px;
    background: var(--app-tint);
  }

  .pack-title {
    border: 0;
    background: transparent;
    color: var(--settings-ink);
    padding: 0;
    text-align: left;
    font-weight: 700;
  }

  .inline-edit {
    width: min(420px, 100%);
    height: 26px;
    border: 1px solid var(--settings-accent);
    border-radius: 5px;
    padding: 0 8px;
  }

  .status {
    display: inline-flex;
    margin-top: 4px;
    border-radius: 999px;
    padding: 2px 7px;
    background: rgba(42, 107, 216, 0.1);
    color: #1f4fa3;
    font-size: 11px;
    font-weight: 700;
    text-transform: capitalize;
  }

  .status.failed {
    background: rgba(196, 48, 32, 0.1);
    color: #a4291a;
  }

  .app-row p,
  .pack-row p {
    margin-top: 2px;
    font-size: 12px;
  }

  .about {
    max-width: 520px;
    margin: 0 auto;
    padding-top: 24px;
    text-align: center;
  }

  .about-mark {
    width: 84px;
    height: 84px;
    display: grid;
    place-items: center;
    margin: 0 auto 20px;
    border-radius: 20px;
    background: linear-gradient(180deg, #d98b3a 0%, #b06a1b 100%);
    color: #fff;
    font-size: 40px;
    font-weight: 800;
    box-shadow:
      0 8px 24px rgba(176, 106, 27, 0.35),
      inset 0 1px 0 rgba(255, 255, 255, 0.25);
  }

  .about > p {
    margin: 14px auto 0;
    max-width: 420px;
  }

  .about-footer {
    margin-top: 36px;
    padding-top: 20px;
    border-top: 0.5px solid var(--settings-line);
    color: var(--settings-ink-3);
    line-height: 1.6;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(2px);
  }

  .modal {
    width: min(520px, calc(100vw - 48px));
    max-height: min(540px, calc(100vh - 48px));
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 0.5px solid rgba(255, 255, 255, 0.4);
    border-radius: 12px;
    background: rgba(246, 243, 236, 0.96);
    box-shadow:
      0 24px 60px rgba(0, 0, 0, 0.32),
      0 0 0 0.5px rgba(0, 0, 0, 0.15);
  }

  .modal-head {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 14px 16px 10px;
    border-bottom: 0.5px solid rgba(0, 0, 0, 0.08);
  }

  .modal-head span {
    color: var(--settings-ink-3);
    font-size: 11.5px;
  }

  .modal-search {
    padding: 10px 12px;
    border-bottom: 0.5px solid rgba(0, 0, 0, 0.06);
  }

  .modal-search .settings-icon {
    width: 12px;
    height: 12px;
    flex-basis: 12px;
  }

  .modal-search input {
    height: 26px;
    border: 0.5px solid rgba(0, 0, 0, 0.18);
    border-radius: 6px;
    background: #fff;
    padding: 0 10px;
  }

  .modal-list {
    flex: 1;
    overflow: auto;
    padding: 6px;
  }

  .picker-row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    padding: 8px 10px;
    color: var(--settings-ink);
    text-align: left;
  }

  .picker-row:hover {
    background: var(--settings-accent);
    color: #fff;
  }

  .picker-row small {
    display: block;
    margin-top: 1px;
    color: var(--settings-ink-3);
  }

  .picker-row:hover small {
    color: rgba(255, 255, 255, 0.8);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    padding: 10px 14px;
    border-top: 0.5px solid rgba(0, 0, 0, 0.08);
  }

  @media (max-width: 820px) {
    .content {
      padding: 28px;
    }

    .sticky-tools {
      top: -28px;
      margin-inline: -28px;
      padding-inline: 28px;
      flex-wrap: wrap;
    }

    .row,
    .step-row,
    .pack-row,
    .app-row {
      gap: 14px;
    }
  }
</style>

<script lang="ts">
  import AppPickerModal from "../components/AppPickerModal.svelte";
  import {
    APP_PICKER_CANDIDATES,
    BUILTIN_APPS,
    createInitialSettingsState,
    type AppIntegration,
  } from "../settings-data";

  let state = createInitialSettingsState();

  let appPickerOpen = false;
  let appSearch = "";

  $: integrationApps = [
    ...BUILTIN_APPS.filter((app) => !state.removedBuiltins.includes(app.id)),
    ...state.customApps,
  ];
  $: pickerItems = getPickerItems(appSearch);

  function getPickerItems(search = "") {
    const removedBuiltins = BUILTIN_APPS.filter((app) => state.removedBuiltins.includes(app.id));
    const visibleIds = new Set(integrationApps.map((app) => app.id));
    const query = search.trim().toLowerCase();

    return [...removedBuiltins, ...APP_PICKER_CANDIDATES].filter(
      (app) =>
        !visibleIds.has(app.id) &&
        (!query || app.name.toLowerCase().includes(query) || app.kind.toLowerCase().includes(query)),
    );
  }

  function setIntegration(id: string, enabled: boolean) {
    state = { ...state, integrations: { ...state.integrations, [id]: enabled } };
  }

  function addApp(app: AppIntegration) {
    const isBuiltin = BUILTIN_APPS.some((candidate) => candidate.id === app.id);

    if (isBuiltin) {
      state = {
        ...state,
        removedBuiltins: state.removedBuiltins.filter((id) => id !== app.id),
        integrations: { ...state.integrations, [app.id]: true },
      };
    } else if (!state.customApps.some((candidate) => candidate.id === app.id)) {
      state = {
        ...state,
        customApps: [...state.customApps, { ...app, custom: true }],
        integrations: { ...state.integrations, [app.id]: true },
      };
    }
  }

  function removeApp(app: AppIntegration) {
    const integrations = { ...state.integrations };
    delete integrations[app.id];

    state = {
      ...state,
      customApps: state.customApps.filter((candidate) => candidate.id !== app.id),
      removedBuiltins: app.custom ? state.removedBuiltins : [...state.removedBuiltins, app.id],
      integrations,
    };
  }

  function selectApp(app: AppIntegration) {
    addApp(app);
    closeAppPicker();
  }

  function closeAppPicker() {
    appPickerOpen = false;
    appSearch = "";
  }
</script>

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
              disabled
              title="Not wired yet"
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
            <p class="result-summary">App integration persistence is not wired yet.</p>

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
                    disabled
                    title="Not wired yet"
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
                    disabled
                    title="Not wired yet"
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
              <button class="button" type="button" disabled title="Not wired yet" on:click={() => (appPickerOpen = true)}>
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
                disabled
                title="Not wired yet"
              >
                {#if state.autoIntegrate}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
              </button>
            </div>
          </div>
        {/if}
      </section>

{#if appPickerOpen}
  <AppPickerModal bind:appSearch {pickerItems} close={closeAppPicker} {selectApp} />
{/if}

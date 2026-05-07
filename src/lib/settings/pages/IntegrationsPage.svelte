<script lang="ts">
  import { onMount } from "svelte";
  import { Client, type Integration } from "$lib/client";
  import { CURATED_INTEGRATION_METADATA } from "../settings-data";

  interface IntegrationRow extends Integration {
    name: string;
    tint: string;
    note?: string;
  }

  let integrations: Integration[] = [];
  let integrationsError = "";
  let isIntegrationsLoading = true;
  let isIntegrationsSaving = false;

  $: integrationApps = integrations.map(toIntegrationRow);

  onMount(() => {
    void loadIntegrations();
  });

  async function loadIntegrations() {
    isIntegrationsLoading = true;
    integrationsError = "";

    try {
      integrations = await Client.getIntegrations();
    } catch (error) {
      integrationsError = `Unable to load integrations: ${error}`;
    } finally {
      isIntegrationsLoading = false;
    }
  }

  function toIntegrationRow(integration: Integration): IntegrationRow {
    const metadata = CURATED_INTEGRATION_METADATA[integration.bundle_id];

    return {
      ...integration,
      name: metadata?.name ?? integration.bundle_id,
      tint: metadata?.tint ?? "#6b6f78",
      note: metadata?.note,
    };
  }

  async function setIntegrationEnabled(bundleId: string, enabled: boolean) {
    const previousIntegrations = integrations;

    integrations = integrations.map((integration) =>
      integration.bundle_id === bundleId ? { ...integration, enabled } : integration,
    );
    isIntegrationsSaving = true;
    integrationsError = "";

    try {
      await Client.setIntegrationEnabled(bundleId, enabled);
    } catch (error) {
      integrations = previousIntegrations;
      integrationsError = `Unable to update integration: ${error}`;
    } finally {
      isIntegrationsSaving = false;
    }
  }

  async function removeIntegration(bundleId: string) {
    const previousIntegrations = integrations;

    integrations = integrations.filter((integration) => integration.bundle_id !== bundleId);
    isIntegrationsSaving = true;
    integrationsError = "";

    try {
      await Client.removeIntegration(bundleId);
    } catch (error) {
      integrations = previousIntegrations;
      integrationsError = `Unable to remove integration: ${error}`;
    } finally {
      isIntegrationsSaving = false;
    }
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
        class="toggle"
        type="button"
        role="switch"
        aria-checked="false"
        aria-label="Toggle watch everywhere"
        disabled
        title="Not wired yet"
      >
        <span></span>
      </button>
    </div>
  </div>

  <div class="divider"></div>

  <div class="stanza">
    <div class="eyebrow">Selected apps</div>
    <p class="section-copy">Harper will only watch the apps you enable here.</p>

    {#if isIntegrationsLoading}
      <p class="result-summary">Loading integrations...</p>
    {:else if integrationsError}
      <p class="result-summary">{integrationsError}</p>
    {:else if isIntegrationsSaving}
      <p class="result-summary">Saving integrations...</p>
    {/if}

    <div class="list-card">
      {#if !isIntegrationsLoading && integrationApps.length === 0}
        <div class="empty">No configured app integrations.</div>
      {:else}
        {#each integrationApps as app}
          <div class="app-row">
            <div class="app-tile" style={`--app-tint: ${app.tint}`}>{app.name[0]}</div>
            <div class="grow">
              <strong>{app.name}</strong>
              <p>{app.bundle_id}</p>
              {#if app.note}<p>{app.note}</p>{/if}
            </div>
            <button
              class="icon-button danger"
              type="button"
              disabled={isIntegrationsLoading || isIntegrationsSaving}
              aria-label={`Remove ${app.name}`}
              on:click={() => removeIntegration(app.bundle_id)}
            >
              <span class="settings-icon icon-trash" aria-hidden="true"></span>
            </button>
            <button
              class:checked={app.enabled}
              class="toggle"
              type="button"
              role="switch"
              disabled={isIntegrationsLoading || isIntegrationsSaving}
              aria-checked={app.enabled}
              aria-label={`Toggle ${app.name}`}
              on:click={() => setIntegrationEnabled(app.bundle_id, !app.enabled)}
            >
              <span></span>
            </button>
          </div>
        {/each}
      {/if}
    </div>

    <div class="actions-row">
      <button class="button" type="button" disabled title="Not wired yet">Add application...</button>
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
        class="checkbox"
        type="button"
        role="checkbox"
        aria-checked="false"
        disabled
        title="Not wired yet"
      ></button>
    </div>
  </div>
</section>

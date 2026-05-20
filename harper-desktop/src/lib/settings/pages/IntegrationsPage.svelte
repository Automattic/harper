<script lang="ts">
import { onMount } from 'svelte';
import { Client, type Integration } from '$lib/client';
import AppPickerModal from '../components/AppPickerModal.svelte';

interface IntegrationRow extends Integration {
	name: string;
}

let integrations: Integration[] = [];
let integrationsError = '';
let isIntegrationsLoading = true;
let isIntegrationsSaving = false;
let appPickerOpen = false;
let newBundleId = '';

$: integrationApps = integrations.map(toIntegrationRow);
$: existingBundleIds = integrations.map((integration) => integration.bundle_id);

onMount(() => {
	void loadIntegrations();
});

async function loadIntegrations() {
	isIntegrationsLoading = true;
	integrationsError = '';

	try {
		integrations = await Client.getIntegrations();
	} catch (error) {
		integrationsError = `Unable to load integrations: ${error}`;
	} finally {
		isIntegrationsLoading = false;
	}
}

function toIntegrationRow(integration: Integration): IntegrationRow {
	return {
		...integration,
		name: integrationName(integration.bundle_id),
	};
}

function integrationName(bundleId: string) {
	return bundleId.split('.').at(-1) || bundleId;
}

async function setIntegrationEnabled(bundleId: string, enabled: boolean) {
	const previousIntegrations = integrations;

	integrations = integrations.map((integration) =>
		integration.bundle_id === bundleId ? { ...integration, enabled } : integration,
	);
	isIntegrationsSaving = true;
	integrationsError = '';

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
	integrationsError = '';

	try {
		await Client.removeIntegration(bundleId);
	} catch (error) {
		integrations = previousIntegrations;
		integrationsError = `Unable to remove integration: ${error}`;
	} finally {
		isIntegrationsSaving = false;
	}
}

async function addIntegration(bundleId: string) {
	const trimmedBundleId = bundleId.trim();

	if (
		!trimmedBundleId ||
		integrations.some((integration) => integration.bundle_id === trimmedBundleId)
	) {
		return;
	}

	const previousIntegrations = integrations;

	integrations = [...integrations, { bundle_id: trimmedBundleId, enabled: true }];
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.addIntegration(trimmedBundleId);
		closeAppPicker();
	} catch (error) {
		integrations = previousIntegrations;
		integrationsError = `Unable to add integration: ${error}`;
	} finally {
		isIntegrationsSaving = false;
	}
}

function closeAppPicker() {
	appPickerOpen = false;
	newBundleId = '';
}
</script>

<section>
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
            <div class="app-tile" style="--app-tint: #6b6f78">{app.name[0]}</div>
            <div class="grow">
              <strong>{app.name}</strong>
              <p>{app.bundle_id}</p>
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
      <button
        class="button"
        type="button"
        disabled={isIntegrationsLoading || isIntegrationsSaving}
        on:click={() => (appPickerOpen = true)}
      >Add application...</button>
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

{#if appPickerOpen}
  <AppPickerModal
    bind:bundleId={newBundleId}
    {existingBundleIds}
    isSaving={isIntegrationsSaving}
    close={closeAppPicker}
    add={addIntegration}
  />
{/if}

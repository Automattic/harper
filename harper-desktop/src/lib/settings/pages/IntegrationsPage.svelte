<script lang="ts">
import { Button, Checkbox, IconButton, Panel, SettingRow, Toggle, TrashIcon } from 'components';
import { onMount } from 'svelte';
import { Client, type Integration } from '$lib/client';
import AppIcon from '../components/AppIcon.svelte';
import AppPickerModal from '../components/AppPickerModal.svelte';

interface IntegrationRow extends Integration {
	name: string;
}

let integrations: Integration[] = [];
let integrationApps: IntegrationRow[] = [];
let integrationsError = '';
let isIntegrationsLoading = true;
let isIntegrationsSaving = false;
let appPickerOpen = false;
let newBundleId = '';

$: existingBundleIds = integrations.map((integration) => integration.bundle_id);

onMount(() => {
	void loadIntegrations();
});

async function loadIntegrations() {
	isIntegrationsLoading = true;
	integrationsError = '';

	try {
		integrations = await Client.getIntegrations();
		integrationApps = integrations.map(toIntegrationRow);
	} catch (error) {
		integrationsError = `Unable to load integrations: ${error}`;
	} finally {
		isIntegrationsLoading = false;
	}
}

function toIntegrationRow(integration: Integration): IntegrationRow {
	return {
		...integration,
		name: integration.display_name,
	};
}

async function setIntegrationEnabled(bundleId: string, enabled: boolean) {
	const previousIntegrations = integrations;
	const previousIntegrationApps = integrationApps;

	integrations = integrations.map((integration) =>
		integration.bundle_id === bundleId ? { ...integration, enabled } : integration,
	);
	integrationApps = integrationApps.map((app) =>
		app.bundle_id === bundleId ? { ...app, enabled } : app,
	);
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.setIntegrationEnabled(bundleId, enabled);
	} catch (error) {
		integrations = previousIntegrations;
		integrationApps = previousIntegrationApps;
		integrationsError = `Unable to update integration: ${error}`;
	} finally {
		isIntegrationsSaving = false;
	}
}

async function removeIntegration(bundleId: string) {
	const previousIntegrations = integrations;
	const previousIntegrationApps = integrationApps;

	integrations = integrations.filter((integration) => integration.bundle_id !== bundleId);
	integrationApps = integrationApps.filter((app) => app.bundle_id !== bundleId);
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.removeIntegration(bundleId);
	} catch (error) {
		integrations = previousIntegrations;
		integrationApps = previousIntegrationApps;
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
	const previousIntegrationApps = integrationApps;

	integrations = [
		...integrations,
		{ bundle_id: trimmedBundleId, enabled: true, display_name: trimmedBundleId },
	];
	integrationApps = integrations.map(toIntegrationRow);
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.addIntegration(trimmedBundleId);
		await loadIntegrations();
		closeAppPicker();
	} catch (error) {
		integrations = previousIntegrations;
		integrationApps = previousIntegrationApps;
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

    <Panel>
      {#if !isIntegrationsLoading && integrationApps.length === 0}
        <div class="empty">No configured app integrations.</div>
      {:else}
        {#each integrationApps as app}
          <div class="app-row">
            <AppIcon bundleId={app.bundle_id} name={app.name} />
            <div class="grow">
              <strong>{app.name}</strong>
              <p>{app.bundle_id}</p>
            </div>
            <IconButton
              danger
              disabled={isIntegrationsLoading || isIntegrationsSaving}
              aria-label={`Remove ${app.name}`}
              on:click={() => removeIntegration(app.bundle_id)}
            >
              <TrashIcon className="control-icon" />
            </IconButton>
            <Toggle
              appearance="settings"
              checked={app.enabled}
              disabled={isIntegrationsLoading || isIntegrationsSaving}
              aria-label={`Toggle ${app.name}`}
              on:click={() => setIntegrationEnabled(app.bundle_id, !app.enabled)}
            />
          </div>
        {/each}
      {/if}
    </Panel>

    <div class="actions-row">
      <Button
        unstyled
        class="button"
        type="button"
        disabled={isIntegrationsLoading || isIntegrationsSaving}
        on:click={() => (appPickerOpen = true)}
      >Add application...</Button>
      <span class="muted">Choose any app from your Applications folder.</span>
    </div>
  </div>

  <div class="divider"></div>

  <div class="stanza">
    <div class="eyebrow">New apps</div>
    <SettingRow top>
      <strong>Enable new apps automatically</strong>
      <p>When you launch a supported app for the first time, turn integration on by default.</p>
      <Checkbox
        slot="control"
        appearance="settings"
        checked={false}
        disabled
        title="Not wired yet"
      />
    </SettingRow>
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

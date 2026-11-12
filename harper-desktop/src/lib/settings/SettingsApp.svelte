<script lang="ts">
import { onMount } from 'svelte';
import { Client, type OnboardingState } from '$lib/client';
import SettingsSidebar from './SettingsSidebar.svelte';
import './settings.css';
import AboutPage from './pages/AboutPage.svelte';
import DictionaryPage from './pages/DictionaryPage.svelte';
import GeneralPage from './pages/GeneralPage.svelte';
import GettingStartedPage from './pages/GettingStartedPage.svelte';
import IntegrationsPage from './pages/IntegrationsPage.svelte';
import RulesPage from './pages/RulesPage.svelte';
import ShortcutsPage from './pages/ShortcutsPage.svelte';
import WeirpacksPage from './pages/WeirpacksPage.svelte';
import WritingPage from './pages/WritingPage.svelte';
import type { SectionId } from './settings-data';

let active: SectionId = 'general';
let contentEl: HTMLElement;
let isLoadingOnboarding = true;
let onboardingState: OnboardingState = 'welcome';
let onboardingError = '';

const titleMap: Record<SectionId, string> = {
	general: 'General',
	writing: 'Writing',
	dictionary: 'Dictionary',
	shortcuts: 'Shortcuts',
	rules: 'Rules',
	weirpacks: 'Weirpacks',
	integrations: 'Integrations',
	about: 'About',
};

onMount(() => {
	void loadOnboardingState();
});

async function loadOnboardingState() {
	isLoadingOnboarding = true;
	onboardingError = '';
	try {
		onboardingState = await Client.getOnboardingState();
	} catch (error) {
		onboardingError = `Unable to load onboarding state: ${error}`;
	} finally {
		isLoadingOnboarding = false;
	}
}

/** Remount the walkthrough at Welcome without clearing persisted completion or other settings. */
function restartOnboarding() {
	active = 'general';
	onboardingCompleted = false;
}

$: title = titleMap[active];

$: if (contentEl && active) {
	contentEl.scrollTop = 0;
}
</script>

{#if isLoadingOnboarding}
  <div class="settings-shell">
    <main class="content" aria-label="Settings">
      <p role="status">Loading settings...</p>
    </main>
  </div>
{:else if onboardingError}
  <div class="settings-shell">
    <main class="content" aria-label="Settings">
      <p role="alert">{onboardingError}</p>
      <button type="button" on:click={loadOnboardingState}>Retry</button>
    </main>
  </div>
{:else if onboardingState !== 'complete'}
  <GettingStartedPage initialState={onboardingState} onComplete={() => (onboardingState = 'complete')} />
{:else}
  <div class="settings-shell">
    <SettingsSidebar bind:active />

    <main bind:this={contentEl} class="content" aria-label={title}>
      {#if active === "general"}
        <GeneralPage onRestartOnboarding={restartOnboarding} />
      {:else if active === "writing"}
        <WritingPage />
      {:else if active === "dictionary"}
        <DictionaryPage />
      {:else if active === "shortcuts"}
        <ShortcutsPage />
      {:else if active === "rules"}
        <RulesPage />
      {:else if active === "weirpacks"}
        <WeirpacksPage />
      {:else if active === "integrations"}
        <IntegrationsPage />
      {:else if active === "about"}
        <AboutPage />
      {/if}
    </main>
  </div>
{/if}

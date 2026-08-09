<script lang="ts">
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
// Accessibility consent settings UI (example component).
// The component lives under $lib/settings and is small; import it below
// where you'd like to render per-app consent controls.
import AccessibilityConsent from '$lib/settings/AccessibilityConsent.svelte';
import type { SectionId } from './settings-data';

let active: SectionId = 'getting-started';
let contentEl: HTMLElement;

const titleMap: Record<SectionId, string> = {
	'getting-started': 'Getting Started',
	general: 'General',
	writing: 'Writing',
	dictionary: 'Dictionary',
	shortcuts: 'Shortcuts',
	rules: 'Rules',
	weirpacks: 'Weirpacks',
	integrations: 'Integrations',
	about: 'About',
};

$: title = titleMap[active];

$: if (contentEl && active) {
	contentEl.scrollTop = 0;
}
</script>

<div class="settings-shell">
  <SettingsSidebar bind:active />

  <main bind:this={contentEl} class="content" aria-label={title}>
    {#if active === "getting-started"}
      <GettingStartedPage navigateToSection={(section) => (active = section)} />
    {:else if active === "general"}
      <GeneralPage />
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
      {!-- Example usage: render a consent control for a known bundle id.
          Uncomment and adapt where useful. --}
      {<!-- <AccessibilityConsent bundleId="com.apple.TextEdit" /> -->}
    {:else if active === "about"}
      <AboutPage />
    {/if}
  </main>
</div>

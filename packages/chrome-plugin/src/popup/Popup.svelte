<script lang="ts">
import logo from '/logo.png';
import type { PopupState } from '../PopupState';
import Main from './Main.svelte';
import Onboarding from './Onboarding.svelte';
import ReportProblematicLint from './ReportProblematicLint.svelte';

let state: PopupState = $state({ page: 'main' });

$effect(() => {
	chrome.storage.local.get({ popupState: { page: 'onboarding' } }).then((result) => {
		state = result.popupState;
	});
});

$effect(() => {
	chrome.storage.local.set({ popupState: state });
});

function openSettings() {
	chrome.runtime?.openOptionsPage?.();
}
</script>

<div class="w-[340px] border border-gray-200 bg-white font-sans flex flex-col rounded-lg shadow-sm select-none">
  <header class="flex items-center gap-2 px-3 py-2 bg-gray-50/60 rounded-t-lg">
    <img src={logo} alt="Harper logo" class="h-6 w-auto" />
    <span class="font-semibold text-sm">Harper</span>
  </header>

  {#if state.page == "onboarding"}
    <Onboarding onConfirm={() => { state = {page: "main"};}} />
  {:else if state.page == "main"}
    <Main /> 
  {:else if state.page == 'report-error'}
    <ReportProblematicLint example={state.example} rule_id={state.rule_id} feedback={state.feedback} />
  {/if}

  <footer class="flex items-center justify-center gap-6 px-3 py-2 text-sm border-t border-gray-100 rounded-b-lg bg-white/60">
    <a href="https://github.com/Automattic/harper" target="_blank" rel="noopener" class="text-primary-600 hover:underline">GitHub</a>
    <a href="https://discord.com/invite/JBqcAaKrzQ" target="_blank" rel="noopener" class="text-primary-600 hover:underline">Discord</a>
    <a href="https://writewithharper.com" target="_blank" rel="noopener" class="text-primary-600 hover:underline">Discover</a>
    <a target="_blank" rel="noopener" class="text-primary-600 hover:underline" on:click={openSettings}>Settings</a>
  </footer>
</div>

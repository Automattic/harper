<script lang="ts">
import { faCaretLeft } from '@fortawesome/free-solid-svg-icons';
import { Button } from 'flowbite-svelte';
import Fa from 'svelte-fa';
import logo from '/logo.png';
import { main, type PopupState } from '../PopupState';
import Main from './Main.svelte';
import Onboarding from './Onboarding.svelte';
import ReportProblematicLint from './ReportProblematicLint.svelte';

let popupState: PopupState = $state({ page: 'main' });

$effect(() => {
	chrome.storage.local.get({ popupState: { page: 'onboarding' } }).then((result) => {
		popupState = result.popupState;
	});
});

$effect(() => {
	chrome.storage.local.set({ popupState: $state.snapshot(popupState) });
});

function openSettings() {
	chrome.runtime?.openOptionsPage?.();
}
</script>

<div class="w-[340px] border border-gray-200 bg-white font-sans flex flex-col rounded-lg shadow-sm select-none">
  <header class="flex flex-row justify-between items-center gap-2 px-3 py-2 bg-gray-50/60 rounded-t-lg">
    <div class="flex flex-row justify-start items-center">
      <img src={logo} alt="Harper logo" class="h-6 w-auto" />
      <span class="font-semibold text-sm">Harper</span>
    </div>

    {#if popupState.page != "main"}
       <Button outline on:click={() => { 
          popupState = main();
       }}><Fa icon={faCaretLeft}/></Button>
    {/if}
  </header>

  {#if popupState.page == "onboarding"}
    <Onboarding onConfirm={() => { popupState = main();}} />
  {:else if popupState.page == "main"}
    <Main /> 
  {:else if popupState.page == 'report-error'}
    <ReportProblematicLint example={popupState.example} rule_id={popupState.rule_id} feedback={popupState.feedback} onSubmit={() => { popupState = main();}} />
  {/if}

  <footer class="flex items-center justify-center gap-6 px-3 py-2 text-sm border-t border-gray-100 rounded-b-lg bg-white/60">
    <a href="https://github.com/Automattic/harper" target="_blank" rel="noopener" class="text-primary-600 hover:underline">GitHub</a>
    <a href="https://discord.com/invite/JBqcAaKrzQ" target="_blank" rel="noopener" class="text-primary-600 hover:underline">Discord</a>
    <a href="https://writewithharper.com" target="_blank" rel="noopener" class="text-primary-600 hover:underline">Discover</a>
    <button class="text-primary-600 hover:underline" onclick={openSettings}>Settings</button>
  </footer>
</div>

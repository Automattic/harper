<script lang="ts">
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons';
import { Button, Link } from 'components';
import { onMount } from 'svelte';
import Fa from 'svelte-fa';
import logo from '/logo.png';
import detectBrowserEngine from '../detectBrowserEngine';
import { main, type PopupState } from '../PopupState';
import DomainReview from './DomainReview.svelte';
import Main from './Main.svelte';
import Onboarding from './Onboarding.svelte';
import ReportProblematicLint from './ReportProblematicLint.svelte';

let popupState: PopupState = $state({ page: 'main' });

let version = `v${chrome.runtime.getManifest().version}`;
let latestVersion: string | null = $state(null);
let versionMismatch = $state(false);

let hoveredFooterIndex = $state<number | null>(null);

onMount(async () => {
	try {
		const response = await fetch('https://writewithharper.com/latestversion');
		if (!response.ok) return;

		const fetchedVersion = (await response.text()).trim();
		latestVersion = fetchedVersion;
		versionMismatch = !!fetchedVersion && fetchedVersion !== version;
	} catch (err) {
		console.error('Failed to fetch latest version', err);
	}
});

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

function openUpdateHelpPage() {
	let url: string;

	if (detectBrowserEngine() == 'chromium') {
		url = 'https://writewithharper.com/docs/integrations/chrome-extension#Updating-the-Extension';
	} else {
		url = 'https://writewithharper.com/docs/integrations/firefox-extension#Updating-the-Extension';
	}

	chrome.tabs.create({
		url,
	});
}

// Per-page heights: compact for the main view, taller for forms.
let heightPx = $derived(popupState.page == 'main' ? 410 : 500);
const WIDTH_PX = 340;

// Browser popups size themselves to the document, and stale CSS (e.g. #app min-h-screen) can keep
// them at the larger size. Setting the size explicitly forces the popup to resize.
$effect(() => {
	const app = document.getElementById('app'); // global CSS gives #app min-h-screen
	for (const el of [document.documentElement, document.body, app]) {
		if (!el) continue;
		el.style.width = `${WIDTH_PX}px`;
		el.style.height = `${heightPx}px`;
		el.style.minHeight = '0';
		el.style.margin = '0';
		el.style.overflow = 'hidden';
	}
});

const footerLink =
	'text-xs text-gray-500 dark:text-slate-400 no-underline! hover:text-primary hover:underline!';
</script>

<div
	style="width: {WIDTH_PX}px; height: {heightPx}px;"
	class="font-sans flex flex-col select-none overflow-hidden rounded-xl border border-gray-200 dark:border-slate-800 dark:text-slate-100"
>
	<header
		class="flex shrink-0 flex-row items-center justify-between gap-2 border-b border-gray-100 px-4 py-3 dark:border-slate-800"
	>
		<div class="flex flex-row items-center gap-2.5">
			<img src={logo} alt="Harper logo" class="h-6 w-6 rounded-md" />
			<span class="text-sm font-semibold tracking-tight">Harper</span>
		</div>

		{#if popupState.page != 'main'}
			<Button
				on:click={() => {
					popupState = main();
				}}
			>
				<Fa icon={faArrowLeft} />
			</Button>
		{:else}
			<button
				type="button"
				class="flex items-center gap-1 rounded-md px-1.5 py-0.5 text-gray-500 transition-colors cursor-pointer hover:bg-slate-800/60 dark:text-slate-400"
				onclick={openUpdateHelpPage}
				title={versionMismatch ? `Newer version available: ${latestVersion ?? ''}. Click to find out more.` : 'How to update'}
			>
				{#if versionMismatch}
					<span class="text-xs" aria-label="Update available">⚠️</span>
				{/if}
				<span class="text-xs font-mono">{version}</span>
			</button>
		{/if}
	</header>

	<div class="min-h-0 flex-1 overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
	{#if popupState.page == 'onboarding'}
		<Onboarding
			onConfirm={() => {
				popupState = main();
			}}
		/>
	{:else if popupState.page == 'main'}
		<Main
			onReviewDomain={(works, domain) => {
				popupState = { page: 'domain-review', works, domain, feedback: '' };
			}}
		/>
	{:else if popupState.page == 'report-error'}
		<ReportProblematicLint
			example={popupState.example}
			rule_id={popupState.rule_id}
			feedback={popupState.feedback}
			onSubmit={() => {
				popupState = main();
			}}
		/>
	{:else if popupState.page == 'domain-review'}
		<DomainReview
			works={popupState.works}
			domain={popupState.domain}
			feedback={popupState.feedback}
			onSubmit={() => {
				popupState = main();
			}}
		/>
	{/if}
	</div>

	<footer
	class="border-t border-gray-100 bg-white/60 p-1.5 dark:border-slate-800 dark:bg-slate-900/70"
	onmouseleave={() => (hoveredFooterIndex = null)}
>
	<div class="relative flex w-full items-center">
		<!-- Gliding Hover Pill -->
		<div
			class="absolute inset-y-0 left-0 w-1/4 rounded-md bg-gray-200/60 transition-all duration-300 ease-out dark:bg-slate-700/50"
			style="transform: translateX(calc({hoveredFooterIndex ?? 0} * 100%)); opacity: {hoveredFooterIndex === null ? 0 : 1};"
		></div>

		<Link 
			href="https://github.com/Automattic/harper" 
			target="_blank" 
			rel="noopener" 
			class="relative z-10 flex-1 rounded-md py-1 text-center text-sm !no-underline transition-colors hover:text-primary dark:text-slate-300 dark:hover:text-primary {footerLink}"
			style="text-decoration: none;"
			onmouseenter={() => (hoveredFooterIndex = 0)}
		>
			GitHub
		</Link>
		
		<Link 
			href="https://discord.com/invite/JBqcAaKrzQ" 
			target="_blank" 
			rel="noopener" 
			class="relative z-10 flex-1 rounded-md py-1 text-center text-sm !no-underline transition-colors hover:text-primary dark:text-slate-300 dark:hover:text-primary {footerLink}"
			style="text-decoration: none;"
			onmouseenter={() => (hoveredFooterIndex = 1)}
		>
			Discord
		</Link>
		
		<Link 
			href="https://writewithharper.com" 
			target="_blank" 
			rel="noopener" 
			class="relative z-10 flex-1 rounded-md py-1 text-center text-sm !no-underline transition-colors hover:text-primary dark:text-slate-300 dark:hover:text-primary {footerLink}"
			style="text-decoration: none;"
			onmouseenter={() => (hoveredFooterIndex = 2)}
		>
			Discover
		</Link>
		
		<Link 
			on:click={openSettings} 
			class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center text-sm !no-underline transition-colors hover:text-primary dark:text-slate-300 dark:hover:text-primary {footerLink}"
			style="text-decoration: none;"
			onmouseenter={() => (hoveredFooterIndex = 3)}
		>
			Settings
		</Link>
	</div>
</footer>
</div>
<script lang="ts">
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons';
import { Button, Link } from 'components';
import { onMount } from 'svelte';
import { fly } from 'svelte/transition';
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
let activeFooterIndex = $state<number>(0);

function handleMouseEnter(index: number) {
	activeFooterIndex = index;
	hoveredFooterIndex = index;
}

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

const footerLink = 'text-xs text-gray-500 dark:text-slate-400 no-underline!';
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
                class="flex cursor-pointer items-center gap-1 rounded-md px-1.5 py-0.5 font-mono text-xs transition-colors
                    {versionMismatch
                        ? 'border border-primary text-primary hover:bg-primary hover:text-black'
                        : 'text-gray-500 hover:text-gray-900 dark:text-slate-400 dark:hover:text-white'}"
                onclick={openUpdateHelpPage}
                title={versionMismatch
                    ? `Newer version available: ${latestVersion ?? ''}. Click to find out more.`
                    : 'How to update'}
            >
                {#if versionMismatch}
                    <span class="text-xs" aria-label="Update available">⚠️</span>
                {/if}
                <span class="text-xs font-mono">{version}</span>
            </button>
        {/if}
    </header>

    <!-- Flex container ensuring children take up full available height -->
    <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
        {#if popupState.page == 'onboarding'}
            <div class="flex h-full flex-col" in:fly={{ y: 12, duration: 250 }}>
                <Onboarding
                    onConfirm={() => {
                        popupState = main();
                    }}
                />
            </div>
        {:else if popupState.page == 'main'}
            <div class="flex h-full flex-col" in:fly={{ y: 12, duration: 250 }}>
                <Main
                    onReviewDomain={(works, domain) => {
                        popupState = { page: 'domain-review', works, domain, feedback: '' };
                    }}
                />
            </div>
        {:else if popupState.page == 'report-error'}
            <div class="flex h-full flex-col" in:fly={{ y: 12, duration: 250 }}>
                <ReportProblematicLint
                    example={popupState.example}
                    rule_id={popupState.rule_id}
                    feedback={popupState.feedback}
                    onSubmit={() => {
                        popupState = main();
                    }}
                />
            </div>
        {:else if popupState.page == 'domain-review'}
            <div class="flex h-full flex-col" in:fly={{ y: 12, duration: 250 }}>
                <DomainReview
                    works={popupState.works}
                    domain={popupState.domain}
                    feedback={popupState.feedback}
                    onSubmit={() => {
                        popupState = main();
                    }}
                />
            </div>
        {/if}
    </div>

    <footer
        class="border-t border-gray-100 bg-white/60 p-1.5 dark:border-slate-800 dark:bg-slate-900/70"
        onmouseleave={() => (hoveredFooterIndex = null)}
    >
        <div class="relative flex w-full items-center">
            <!-- Gliding Orange Hover Pill -->
            <div
                class="absolute inset-y-0 left-0 w-1/4 rounded-md bg-primary transition-all duration-300 ease-out"
                style="transform: translateX(calc({activeFooterIndex} * 100%)); opacity: {hoveredFooterIndex === null ? 0 : 1};"
            ></div>

            <Link 
                href="https://github.com/Automattic/harper" 
                target="_blank" 
                rel="noopener" 
                class="relative z-10 flex-1 rounded-md py-1 text-center font-medium no-underline! transition-colors duration-200 {footerLink} {hoveredFooterIndex === 0 ? 'text-black! font-semibold' : ''}"
                style="text-decoration: none;"
                onmouseenter={() => handleMouseEnter(0)}
            >
                GitHub
            </Link>
            
            <Link 
                href="https://discord.com/invite/JBqcAaKrzQ" 
                target="_blank" 
                rel="noopener" 
                class="relative z-10 flex-1 rounded-md py-1 text-center font-medium no-underline! transition-colors duration-200 {footerLink} {hoveredFooterIndex === 1 ? 'text-black! font-semibold' : ''}"
                style="text-decoration: none;"
                onmouseenter={() => handleMouseEnter(1)}
            >
                Discord
            </Link>
            
            <Link 
                href="https://writewithharper.com" 
                target="_blank" 
                rel="noopener" 
                class="relative z-10 flex-1 rounded-md py-1 text-center font-medium no-underline! transition-colors duration-200 {footerLink} {hoveredFooterIndex === 2 ? 'text-black! font-semibold' : ''}"
                style="text-decoration: none;"
                onmouseenter={() => handleMouseEnter(2)}
            >
                Discover
            </Link>
            
            <Link 
                on:click={openSettings} 
                class="relative z-10 flex-1 cursor-pointer rounded-md py-1 text-center font-medium no-underline! transition-colors duration-200 {footerLink} {hoveredFooterIndex === 3 ? 'text-black! font-semibold' : ''}"
                style="text-decoration: none;"
                onmouseenter={() => handleMouseEnter(3)}
            >
                Settings
            </Link>
        </div>
    </footer>
</div>
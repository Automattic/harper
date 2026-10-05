<script lang="ts">
import { Button } from 'components';
import generateGreeting from '../generateGreeting';
import ProtocolClient from '../ProtocolClient';

let { onReviewDomain }: { onReviewDomain: (works: boolean, domain: string) => void } = $props();

let enabled = $state(true);
let domain = $state('');
let site = $derived(domain || 'this page');

let installDate: Date | null = $state(null);
let hasBeenReviewed: boolean | null = $state(null);
const REVIEW_URL =
	'https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews';

const isFirefox = isFirefoxExtension();

let hoveredFooterIndex = $state<number | null>(null);

if (!isFirefox) {
	ProtocolClient.getInstalledOn().then((d) => {
		if (d == null) {
			return;
		}

		installDate = new Date(d);
	});

	ProtocolClient.getReviewed().then((r) => {
		hasBeenReviewed = r;
	});
}

getCurrentTabDomain().then((d) => {
	domain = d ?? '';
});

$effect(() => {
	ProtocolClient.getDomainEnabled(domain).then((e) => {
		enabled = e;
	});
});

/**
 * Returns the registrable domain (e.g.  "example.com") of the
 * tab that the user had open when they clicked the extension icon.
 * If the URL is unavailable (about:blank, chrome://…) it resolves to undefined.
 */
export async function getCurrentTabDomain(): Promise<string | undefined> {
	const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

	if (!tab?.url) return undefined;

	try {
		const { hostname } = new URL(tab.url);
		return hostname.replace(/^www\./, '');
	} catch {
		return undefined;
	}
}

function toggleDomainEnabled() {
	enabled = !enabled;
	ProtocolClient.setDomainEnabled(domain, enabled);
}

function openReviewPage() {
	ProtocolClient.setReviewed(true);
	chrome.tabs.create({ url: REVIEW_URL });
}

async function reviewDomain(works: boolean) {
	onReviewDomain(works, (await getCurrentTabDomain()) ?? 'unknown');
}

function isFirefoxExtension(): boolean {
	try {
		return new URL(chrome.runtime.getURL('')).protocol === 'moz-extension:';
	} catch {
		return false;
	}
}

/** Get the number of days since a given Date. */
function daysSince(date: Date): number {
	let now = Date.now();
	let then = date.getTime();

	let msDiff = now - then;
	return msDiff / 86400000;
}
</script>

<main class="flex h-full flex-col">
	<section
		class="flex flex-1 flex-col justify-center px-5 py-6 text-center transition-colors"
		style={enabled
			? 'background-image: radial-gradient(circle at 50% 45%, color-mix(in srgb, var(--color-primary) 12%, transparent), transparent 75%)'
			: ''}
	>
		<!-- Vertical Stack Layout -->
		<div class="flex w-full flex-col items-center gap-5">
			
			<!-- 1. Greeting (Full Width, No Truncation) -->
			<h2 class="text-2xl font-medium dark:text-white">{generateGreeting()}</h2>
			
			<!-- 2. Dedicated Status Bar (Full Width for Long Domains) -->
			<div 
				class="inline-flex max-w-full items-center gap-1.5 rounded-full border border-gray-200/60 bg-gray-50/50 px-3 py-1 text-xs text-gray-600 dark:border-slate-700/60 dark:bg-slate-800/40 dark:text-slate-400" 
				title={site}
			>
				<span
					class="h-2 w-2 shrink-0 rounded-full {enabled ? 'bg-emerald-400 shadow-[0_0_6px_1px_rgba(52,211,153,0.4)]' : 'bg-slate-500'}"
					aria-hidden="true"
				></span>
				<span class="text-gray-600 dark:text-gray-400 shrink-0">{enabled ? 'Checking on' : 'Paused on'}</span>
				<span class="truncate font-medium text-gray-900 dark:text-white">{site}</span>
			</div>

			<!-- 3. Animated On/Off Sliding Toggle -->
			<div
				class="relative flex w-48 shrink-0 rounded-lg border border-gray-200 p-1 shadow-sm dark:border-slate-700 dark:bg-slate-900"
				role="group"
				aria-label="Toggle Harper"
			>
				<div
					class="absolute inset-y-1 left-1 w-[calc(50%-0.25rem)] rounded-md bg-primary transition-transform duration-300 ease-out"
					class:translate-x-0={enabled}
					class:translate-x-full={!enabled}
				></div>

				<button
					type="button"
					aria-pressed={enabled}
					class="relative z-10 flex-1 cursor-pointer rounded-md py-1.5 text-sm font-medium transition-colors duration-300 {enabled ? 'text-black' : 'text-gray-500 hover:text-white dark:text-slate-400'}"
					onclick={() => { if (!enabled) toggleDomainEnabled(); }}
				>
					On
				</button>
				<button
					type="button"
					aria-pressed={!enabled}
					class="relative z-10 flex-1 cursor-pointer rounded-md py-1.5 text-sm font-medium transition-colors duration-300 {!enabled ? 'text-black' : 'text-gray-500 hover:text-white dark:text-slate-400'}"
					onclick={() => { if (enabled) toggleDomainEnabled(); }}
				>
					Off
				</button>
			</div>

		</div>
	</section>

	{#if !isFirefox && installDate != null && daysSince(installDate) > 14 && hasBeenReviewed === false}
		<section class="flex shrink-0 flex-row items-center justify-between gap-3 bg-primary px-5 py-3 text-black">
			<div class="text-sm font-semibold leading-snug">
				Enjoying Harper? A quick review helps a lot.
			</div>
			<Button on:click={openReviewPage}>Review</Button>
		</section>
	{:else}
		<!-- Compact Side-by-Side Review Section -->
		<section
			class="flex shrink-0 flex-row items-center justify-between gap-3 border-t border-gray-100 px-5 py-3 dark:border-slate-800 dark:bg-slate-900/50"
		>
			<p class="text-sm font-medium leading-tight dark:text-slate-200">Does Harper work well here?</p>

			<div class="flex w-24 shrink-0 flex-row rounded-md border border-gray-200 p-0.5 dark:border-slate-700 dark:bg-slate-900">
				<button
					type="button"
					aria-label="Harper works well on this site"
					class="flex-1 cursor-pointer rounded-sm py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-primary hover:text-black dark:text-slate-400 dark:hover:text-black"
					onclick={() => reviewDomain(true)}
				>
					Yes
				</button>
				<button
					type="button"
					aria-label="Harper has problems on this site"
					class="flex-1 cursor-pointer rounded-sm py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-primary hover:text-black dark:text-slate-400 dark:hover:text-black"
					onclick={() => reviewDomain(false)}
				>
					No
				</button>
			</div>
		</section>
	{/if}
</main>
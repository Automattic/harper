<script lang="ts">
import { Button } from 'components';
import generateGreeting from '../generateGreeting';
import ProtocolClient from '../ProtocolClient';

let enabled = $state(true);
let domain = $state('');

let installDate: Date | null = $state(null);
let hasBeenReviewed: boolean | null = $state(null);
const REVIEW_URL =
	'https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews';

ProtocolClient.getInstalledOn().then((d) => {
	if (d == null) {
		return;
	}

	installDate = new Date(d);
});

ProtocolClient.getReviewed().then((r) => {
	hasBeenReviewed = r;
});

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
		return hostname.replace(/^www\\./, '');
	} catch {
		return undefined;
	}
}

function toggleDomainEnabled() {
	console.log('toggle');
	enabled = !enabled;
	ProtocolClient.setDomainEnabled(domain, enabled);
}

function openReviewPage() {
	ProtocolClient.setReviewed(true);
	chrome.tabs.create({ url: REVIEW_URL });
}

/** Get the number of days since a given Date. */
function daysSince(date: Date): number {
	let now = Date.now();
	let then = date.getTime();

	let msDiff = now - then;
	return msDiff / 86400000;
}
</script>

<main>
  <section class="p-6 space-y-5 text-gray-800 flex flex-row">
      <Button
        size="lg"
        class="rounded-full! aspect-square h-24 w-24 p-0 shadow-lg transition-colors flex! flex-row justify-center"
        color={enabled ? 'var(--color-primary)' : 'var(--color-cream-50)'}
        on:click={toggleDomainEnabled}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-9 w-9"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
          color={enabled ? 'var(--color-cream-50)' : 'var(--color-primary)'}
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 5v7m5.657-4.657a8 8 0 11-11.314 0"
          />
        </svg>
      </Button>
  
    <section class="items-end p-2">
      <h1 class="text-2xl dark:text-white text-right">
        {generateGreeting()}
      </h1>
  
      <p class="text-sm font-medium font-sans dark:text-white text-right">
        Harper is {enabled ? 'enabled on ' : 'disabled on '}{domain}
      </p>
    </section>
  </section>
  
  {#if installDate != null && daysSince(installDate) > 7 && hasBeenReviewed === false}
    <section class="bg-primary flex flex-row justify-between p-4">
      <div class="font-bold">
        It looks like you're enjoying Harper.<br>
        Would you mind giving us a review?
      </div>
      <Button on:click={openReviewPage}>
        Review
      </Button>
    </section>
  {/if}
</main>

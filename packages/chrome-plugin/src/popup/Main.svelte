<script lang="ts">
import { Button, Select, Toggle } from 'flowbite-svelte';
import ProtocolClient from '../ProtocolClient';

let enabled = $state(true);
let domain = $state('');

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
</script>

<main class="p-6 space-y-5 text-gray-800">
  <!-- power button section -->
  <section class="flex flex-row items-center gap-3 py-6">
    <Button
      size="lg"
      class="rounded-full aspect-square h-16 w-16 p-0 shadow-md transition-colors flex flex-row justify-center"
      style="background-color: {enabled ? 'var(--color-primary-500)' : '#d1d5db'};"
      on:click={toggleDomainEnabled}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-9 w-9 text-white"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        stroke-width="2"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 5v7m5.657-4.657a8 8 0 11-11.314 0"
        />
      </svg>
    </Button>

    <p class="text-sm font-medium">
      {enabled ? 'Enabled on ' : 'Disabled on '}{domain}
    </p>
  </section>
</main>

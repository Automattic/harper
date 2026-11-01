<!--
@component
Shows the current browser extension before Harper Desktop. Chrome is the fallback
for server rendering and unsupported browsers; detection runs after mounting.
Pass campaign to set utm_campaign on each browser extension's store URL, preserving
other query parameters. Non-browser links are unchanged. The parent must call
loadLiveVersions(); this component reads the shared store without fetching again.
-->
<script lang="ts">
import { Arrow } from 'components';
import { onMount } from 'svelte';
import getBrowserType from '$lib/getBrowserType';
import { type Integration, integrations } from './data';
import IntegrationTile from './IntegrationTile.svelte';
import { isUpToDate, liveVersions } from './versions';

export let campaign: string;

let browserType: ReturnType<typeof getBrowserType> = 'chrome';

const VERSION_UPTODATE = 'Up-to-date';
const VERSION_BEHIND = 'This version is slightly behind the core engine due to a delay';

onMount(() => {
	browserType = getBrowserType();
});

/** Sets the browser store campaign, preserving other parameters and non-browser links. */
function integrationHref(integration: Integration, campaign: string): string {
	if (integration.category !== 'browsers') {
		return integration.href;
	}

	const url = new URL(integration.href);
	url.searchParams.set('utm_campaign', campaign);
	return url.toString();
}
</script>

<div class="mx-auto grid w-full max-w-[47.5rem] grid-cols-2 gap-[0.85rem] text-left max-[640px]:grid-cols-1">
	{#each [browserType, 'desktop'] as id}
		{@const integration = integrations.find((item) => item.id === id)}
		{#if integration}
			{@const upToDate = isUpToDate($liveVersions.harper, $liveVersions[integration.id])}
			{@const titleText = upToDate ? VERSION_UPTODATE : upToDate === false ? VERSION_BEHIND : null}
			<a
				class="grid grid-cols-[2.5rem_1fr_auto] items-center gap-[0.9rem] rounded-xl border-[0.5px] border-[rgba(28,26,22,0.1)] bg-white px-[1.1rem] py-[0.9rem] !text-[#1c1a16] no-underline transition-[transform,box-shadow,border-color] duration-150 hover:-translate-y-px hover:border-[#b06a1b] hover:shadow-[0_10px_24px_-16px_rgba(28,26,22,0.16)] hover:no-underline dark:border-white/10 dark:bg-white/5 dark:!text-white dark:hover:border-primary-300 max-[640px]:grid-cols-[2.5rem_1fr] [&_em]:max-[640px]:col-start-2"
				href={integrationHref(integration, campaign)}
			>
				<IntegrationTile {integration} size={40} />
				<span class="flex min-w-0 flex-col">
					<div class="flex items-center gap-1.5">
						<strong class="text-[0.94rem] leading-[1.25]">{integration.name}</strong>
						{#if $liveVersions[integration.id] && upToDate != null}
							<span
								class="inline-flex items-center rounded-full bg-[#f4f1ea] dark:bg-white/10 px-1.5 py-0.5 text-[0.68rem] font-mono font-medium text-[#6b6455] dark:text-white/80 border border-[#e4dfd3] dark:border-white/10 select-none"
								title={titleText}
							>
								{$liveVersions[integration.id]}
							</span>
						{/if}
					</div>
					<small class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.8rem] leading-[1.4] text-[#807a6e] dark:text-white/55">
						{integration.desc}
					</small>
				</span>
				<em class="inline-flex items-center gap-1 whitespace-nowrap text-[0.78rem] font-extrabold text-[#b06a1b] not-italic dark:text-primary-300">
					{integration.cta === 'install' ? 'Install' : 'View docs'}<Arrow width="1rem" height="1rem" />
				</em>
			</a>
		{/if}
	{/each}
</div>

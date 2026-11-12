<script lang="ts">
import { page } from '$app/state';
import type { SidebarEntry } from './config';

/** Ordered sidebar sections supplied by the matching route configuration. */
interface Props {
	entries: SidebarEntry[];
}

let { entries }: Props = $props();
const pathname = $derived(page.url.pathname.replace(/\/$/, ''));
</script>

<nav class="text-sm leading-[1.5] [&_a]:my-px [&_a]:block [&_a]:rounded-md [&_a]:px-3 [&_a]:py-1.5 [&_a]:leading-5 [&_a:hover]:bg-(--docs-border) [&_a[aria-current]]:bg-[color-mix(in_srgb,var(--docs-primary)_8%,transparent)] dark:[&_a[aria-current]]:bg-[color-mix(in_srgb,var(--docs-primary)_12%,transparent)]" aria-label="Documentation">
	{#each entries as entry}
		{#if 'items' in entry}
			<section class="border-b border-(--docs-border) py-3">
				{#if entry.title}<h2 class="m-0 mb-2! pl-3 [font-family:inherit]! text-sm! font-semibold! leading-[1.5]! tracking-normal! text-(--docs-heading)!">{entry.title}</h2>{/if}
				<ul class="m-0 list-none! p-0!">
					{#each entry.items as link}
						<li class="m-0"><a href={link.to} aria-current={pathname === link.to ? 'page' : undefined}>{link.title}</a></li>
					{/each}
				</ul>
			</section>
		{:else}
			<a class="mt-3! font-semibold" href={entry.to} aria-current={pathname === entry.to ? 'page' : undefined}>{entry.title}</a>
		{/if}
	{/each}
</nav>

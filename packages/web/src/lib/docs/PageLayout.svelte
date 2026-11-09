<script lang="ts">
import { onMount, type Snippet } from 'svelte';
import { page } from '$app/state';
import { sidebar, siteConfig } from './config';
import TableOfContents, { type Heading } from './TableOfContents.svelte';

/** Page content and optional metadata. Source paths are relative to src/routes and include the filename. */
interface Props {
	children: Snippet;
	title?: string;
	sourcePath?: string;
}

let { children, title, sourcePath }: Props = $props();
let content: HTMLDivElement;
let headings: Heading[] = $state([]);
const links = $derived(
	Object.entries(sidebar)
		.filter(([prefix]) => page.url.pathname.startsWith(prefix))
		.flatMap(([, entries]) =>
			entries.flatMap((entry) => ('items' in entry ? entry.items : [entry])),
		),
);
const index = $derived(links.findIndex((link) => link.to === page.url.pathname.replace(/\/$/, '')));
const previous = $derived(index > 0 ? links[index - 1] : undefined);
const next = $derived(index >= 0 ? links[index + 1] : undefined);
const editLink = $derived(
	sourcePath ? siteConfig.editLink.replace(':route', sourcePath) : undefined,
);

/** Read Markdown anchors without including headings inside embedded UI or changing their IDs. */
function collectHeadings() {
	headings = Array.from(
		content.querySelectorAll<HTMLElement>(
			'[data-docs-content] :is(h2[id], h3[id]):not(:where([data-docs-embed], [data-docs-embed] *))',
		),
	).map((heading) => ({
		id: heading.id,
		text: heading.textContent?.replace(/\s*#\s*$/, '').trim() ?? '',
		level: Number(heading.tagName.slice(1)),
	}));
}

onMount(() => {
	collectHeadings();
	const observer = new MutationObserver(collectHeadings);
	observer.observe(content, {
		childList: true,
		subtree: true,
		characterData: true,
		attributes: true,
		attributeFilter: ['id'],
	});
	return () => observer.disconnect();
});
</script>

<div class="mx-auto w-[45vw] max-w-[780px] pb-28 max-[949px]:w-[90vw] max-[949px]:pb-8">
	{#if headings.length}
		<details class="mb-6 hidden border-b border-(--docs-border) py-2 text-sm leading-[1.75] max-[949px]:block! [&_summary]:cursor-pointer [&_nav]:my-3 [&_h2]:hidden">
			<summary>On this page</summary>
			<TableOfContents {headings} />
		</details>
	{/if}
	<div bind:this={content}>
		{#if title}<h1 class="mt-0 mb-5 text-[32px]! leading-[1.3]! font-bold! tracking-[-0.02em]! text-(--docs-heading)!">{title}</h1>{/if}
		{@render children()}
	</div>
	{#if editLink}
		<div class="mt-16 text-sm leading-[1.75] [&_a]:text-(--docs-link)! [&_a:hover]:underline"><a href={editLink}>Edit this page on GitHub ↗</a></div>
	{/if}
	{#if previous || next}
		<nav class="mt-6 grid grid-cols-2 gap-4 [&_a]:block [&_a]:h-full [&_a]:rounded-lg [&_a]:border [&_a]:border-(--docs-border) [&_a]:px-4 [&_a]:py-3 [&_a]:text-sm [&_a]:leading-[1.75] [&_a]:text-(--docs-link)! [&_span]:block [&_span]:text-xs [&_span]:leading-[1.75] [&_span]:text-(--docs-muted) [&>div:last-child]:text-right" aria-label="Previous and next pages">
			<div>{#if previous}<a href={previous.to}><span>Previous page</span>{previous.title}</a>{/if}</div>
			<div>{#if next}<a href={next.to}><span>Next page</span>{next.title}</a>{/if}</div>
		</nav>
	{/if}
	<aside class="fixed top-[105px] right-[max(24px,calc((100vw-1440px)/2+24px))] max-h-[calc(100vh-130px)] w-[min(18vw,232px)] overflow-y-auto max-[949px]:hidden!"><TableOfContents {headings} /></aside>
</div>

<script module lang="ts">
/** An existing heading anchor; IDs are supplied by the Markdown processor, not this component. */
export interface Heading {
	id: string;
	text: string;
	level: number;
}
</script>

<script lang="ts">
/** Heading links in document order, shared by desktop and mobile outlines. */
interface Props {
	headings: Heading[];
}

let { headings }: Props = $props();
let activeId = $state('');

// Reattach when route content changes, and remove the listener on unmount.
$effect(() => {
	const elements = headings.map((heading) => document.getElementById(heading.id));
	const update = () => {
		let current = headings[0]?.id ?? '';
		for (const element of elements) {
			if (element && element.getBoundingClientRect().top <= 130) current = element.id;
		}
		activeId = current;
	};
	update();
	window.addEventListener('scroll', update, { passive: true });
	return () => window.removeEventListener('scroll', update);
});
</script>

{#if headings.length}
	<nav class="text-sm leading-[2]" aria-label="On this page">
		<h2 class="m-0 mb-1! [font-family:inherit]! text-sm! leading-[1.2]! font-semibold! tracking-normal! text-(--docs-heading)!">On this page</h2>
		<ul class="m-0 list-none! border-l border-(--docs-border) py-0! pr-0! pl-4!">
			{#each headings as heading}
				<li class="m-0" style:padding-left={`${Math.max(0, heading.level - 2) * 12}px`}>
					<a class="block text-(--docs-muted)!" href={`#${heading.id}`} aria-current={activeId === heading.id ? 'location' : undefined}>{heading.text}</a>
				</li>
			{/each}
		</ul>
	</nav>
{/if}

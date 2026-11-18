<script module lang="ts">
import { docSearchConfig } from './config';

// Browser mounts alone initialize these values; SSR leaves them untouched. The library has no
// public disposer, so retain one renderer and its listeners for the document's lifetime.
let mount: HTMLDivElement | undefined;
let loading: Promise<void> | undefined;
let owner: symbol | undefined;
let restoreFocus = true;

/** Close through library controls and suppress focus restoration during navigation or release. */
function closeSearch() {
	restoreFocus = false;
	if (document.querySelector('.DocSearch-Container')) {
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape' }));
		// Opening can precede the library's deferred keyboard effect.
		document.querySelector<HTMLButtonElement>('.DocSearch-Cancel')?.click();
	}
}

/**
 * Borrow the persistent renderer for this component. Late imports and obsolete cleanup cannot
 * reclaim a newer attachment. Release disconnects this observer and retains the detached mount
 * for the next header; it does not try to dispose the library's document-lifetime listeners.
 */
function attachSearch(container: HTMLElement, onFailure: () => void): () => void {
	if (owner) closeSearch();
	const attachment = Symbol();
	owner = attachment;
	mount ??= document.createElement('div');
	container.append(mount);

	let wasOpen = !!document.querySelector('.DocSearch-Container');
	const observer = new MutationObserver(() => {
		const isOpen = !!document.querySelector('.DocSearch-Container');
		if (owner === attachment) {
			if (isOpen && !wasOpen) restoreFocus = true;
			if (wasOpen && !isOpen && restoreFocus) {
				const button = mount?.querySelector<HTMLButtonElement>('.DocSearch-Button');
				if (button?.isConnected && button.getClientRects().length) button.focus();
			}
		}
		wasOpen = isOpen;
	});
	observer.observe(document.body, { childList: true });

	loading ??= import('@docsearch/js').then(({ default: docsearch }) => {
		if (mount) docsearch({ container: mount, ...docSearchConfig });
	});
	void loading.catch(() => {
		if (owner === attachment) onFailure();
	});

	return () => {
		observer.disconnect();
		if (owner !== attachment) return;
		closeSearch();
		owner = undefined;
		mount?.remove();
	};
}

// Svelte HMR must not create another renderer while the old library listeners still exist.
if (import.meta.hot) import.meta.hot.dispose(() => window.location.reload());
</script>

<script lang="ts">
import '@docsearch/css/dist/style.css';
import { onMount } from 'svelte';
import { beforeNavigate } from '$app/navigation';

/** Select the header appearance without changing search configuration or renderer ownership. */
interface Props {
	variant?: 'docs' | 'marketing';
}

let { variant = 'docs' }: Props = $props();
let container: HTMLDivElement;
let failed = $state(false);

beforeNavigate(closeSearch);
onMount(() => attachSearch(container, () => { failed = true; }));

const docsClasses = `min-w-10 flex-1
	[&_.DocSearch-Button]:m-0! [&_.DocSearch-Button]:h-9! [&_.DocSearch-Button]:w-full! [&_.DocSearch-Button]:max-w-[260px]! [&_.DocSearch-Button]:rounded-lg! [&_.DocSearch-Button]:border! [&_.DocSearch-Button]:border-(--docs-border)! [&_.DocSearch-Button]:bg-(--docs-panel)! [&_.DocSearch-Button]:px-3! [&_.DocSearch-Button]:text-(--docs-muted)! [&_.DocSearch-Button]:shadow-none! [&_.DocSearch-Button:hover]:border-(--docs-link)!
	[&_.DocSearch-Button-Placeholder]:text-[13px]! [&_.DocSearch-Search-Icon]:size-4! [&_.DocSearch-Search-Icon]:text-inherit!
	[&_.DocSearch-Button-Key]:border! [&_.DocSearch-Button-Key]:border-(--docs-border)! [&_.DocSearch-Button-Key]:bg-transparent! [&_.DocSearch-Button-Key]:text-(--docs-muted)! [&_.DocSearch-Button-Key]:shadow-none! [&_.DocSearch-Button-Key]:[font-family:inherit]!
	max-[949px]:[&_.DocSearch-Button]:ml-auto! max-[479px]:[&_.DocSearch-Button-Keys]:hidden!`;

const marketingClasses = `dark:[&_.DocSearch-Button]:bg-white/10! dark:[&_.DocSearch-Button]:text-zinc-300! dark:[&_.DocSearch-Button]:border-white/20! dark:[&_.DocSearch-Button:hover]:bg-white/20! dark:[&_.DocSearch-Search-Icon]:text-zinc-300!
	dark:[&_.DocSearch-Button-Keys]:bg-transparent! dark:[&_.DocSearch-Button-Keys]:border-white/20! dark:[&_.DocSearch-Button-Key]:bg-transparent! dark:[&_.DocSearch-Button-Key]:text-zinc-300! dark:[&_.DocSearch-Button-Key]:border-0!`;
</script>

<!-- The modal is portaled to body, so both variants set its dark colors on the root. -->
<div
	class:marketing-docsearch={variant === 'marketing'}
	class="{variant === 'marketing' ? marketingClasses : docsClasses}
		[html.dark:has(&)]:[--docsearch-text-color:#dfdedd]! [html.dark:has(&)]:[--docsearch-muted-color:#a1a1aa]! [html.dark:has(&)]:[--docsearch-searchbox-focus-background:#18181b]! [html.dark:has(&)]:[--docsearch-modal-background:#202023]! [html.dark:has(&)]:[--docsearch-footer-background:#1c1c1f]! [html.dark:has(&)]:[--docsearch-hit-background:#2a2a2e]! [html.dark:has(&)]:[--docsearch-hit-color:#dfdedd]! [html.dark:has(&)]:[--docsearch-key-gradient:linear-gradient(#2a2a2e,#2a2a2e)]! [html.dark:has(&)]:[--docsearch-key-shadow:none]! [html.dark:has(&)]:[--docsearch-footer-shadow:none]! [html.dark:has(&)]:[--docsearch-container-background:rgb(9_10_17/80%)]!"
>
	<div bind:this={container}></div>
	{#if failed}<p role="status" class="text-sm">Search unavailable. <a href="/docs/about" class="underline!">Browse documentation</a>.</p>{/if}
</div>

<style>
.marketing-docsearch {
	min-width: 11rem;
}

:global(.marketing-docsearch .DocSearch-Button) {
	height: 2.5rem;
	margin: 0;
	border: 0.5px solid var(--marketing-line);
	border-radius: 999px;
	background: rgba(28, 26, 22, 0.04);
	color: var(--marketing-ink-3);
	box-shadow: none;
	font-family: inherit;
	padding: 0 0.75rem;
}

:global(.marketing-docsearch .DocSearch-Button:focus-visible) {
	outline: 2px solid var(--docsearch-primary-color);
	outline-offset: 2px;
}

:global(.marketing-docsearch .DocSearch-Button:hover) {
	background: rgba(28, 26, 22, 0.07);
	box-shadow: none;
	color: var(--marketing-ink);
}

:global(.marketing-docsearch .DocSearch-Button-Placeholder) {
	padding: 0 0.65rem 0 0.15rem;
	font-size: 1rem;
	font-weight: 400;
}

:global(.marketing-docsearch .DocSearch-Search-Icon) {
	width: 1.05rem;
	height: 1.05rem;
	color: var(--marketing-ink-3);
	stroke-width: 2;
}

:global(.marketing-docsearch .DocSearch-Button-Keys) {
	display: inline-flex;
	align-items: center;
	min-width: 2.25rem;
	height: 1.35rem;
	justify-content: center;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.3rem;
	background: #fff;
	color: var(--marketing-ink-2);
	font-family: "JetBrains Mono", monospace;
	font-size: 0.68rem;
	font-weight: 700;
}

:global(.marketing-docsearch .DocSearch-Button-Key) {
	display: inline-flex;
	top: 0;
	width: auto;
	min-width: 1.35rem;
	height: 1.25rem;
	padding: 0 0.32rem;
	border: 0.5px solid var(--marketing-line);
	border-radius: 0.25rem;
	background: #fff;
	box-shadow: none;
	color: var(--marketing-ink-2);
	font-family: "JetBrains Mono", monospace;
	font-size: 0.62rem;
}

:global(.DocSearch-SearchBar) {
	padding-bottom: calc(var(--docsearch-spacing) / 2);
}

:global(.DocSearch-Form:focus-within) {
	outline: 2px solid #2a6bd8;
	outline-offset: 2px;
	box-shadow: none;
}

:global(.DocSearch-Input:focus),
:global(.DocSearch-Input:focus-visible) {
	border: 0 !important;
	outline: 0 !important;
	box-shadow: none !important;
	-webkit-appearance: none;
	appearance: none;
}

@media (max-width: 760px) {
	.marketing-docsearch {
		min-width: 0;
	}

	:global(.marketing-docsearch .DocSearch-Button) {
		width: 2rem;
		padding: 0;
	}

	:global(.marketing-docsearch .DocSearch-Button-Placeholder),
	:global(.marketing-docsearch .DocSearch-Button-Keys) {
		display: none;
	}
}
</style>

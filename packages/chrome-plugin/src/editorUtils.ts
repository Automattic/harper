export function findAncestor(
	el: HTMLElement,
	predicate: (el: HTMLElement) => boolean,
): HTMLElement | null {
	let node = el.parentElement;

	while (node != null) {
		if (predicate(node)) {
			return node;
		}

		node = node.parentElement;
	}

	return null;
}

/** Determines if a given node is a child of a P2 editor instance.
 * If so, returns the root node of that instance. */
export function getP2Root(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('p2-editor'));
}

/** Determines if a given node is a child of a Lexical editor instance.
 * If so, returns the root node of that instance. */
export function getLexicalRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(
		el,
		(node: HTMLElement) => node.getAttribute('data-lexical-editor') == 'true',
	);
}

/** Determines if a given node is a child of a Slate.js editor instance.
 * If so, returns the root node of that instance. */
export function getSlateRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.getAttribute('data-slate-editor') == 'true');
}

/** Determines if a given node is a child of a Trix editor instance.
 * If so, returns the root node of that instance. */
export function getTrixRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.nodeName == 'TRIX-EDITOR');
}

/** Determines if a given node is a child of a Reddit composer instance.
 * If so, returns the root node of that instance. */
export function getShredditComposerRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.nodeName == 'SHREDDIT-COMPOSER');
}

/** Determines if a given node is a child of a Quill.js editor instance.
 * If so, returns the root node of that instance. */
export function getQuillJsRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('ql-container'));
}

/** Determines if a given node is a child of a Medium.com editor instance.
 * If so, returns the root node of that instance. */
export function getMediumRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(
		el,
		(node: HTMLElement) => node.nodeName == 'MAIN' && location.hostname == 'medium.com',
	);
}

/** Determines if a given node is a child of a Notion editor instance.
 * If so, returns the root node of that instance. */
export function getNotionRoot(el: HTMLElement): HTMLElement | null {
	return document.getElementById('notion-app');
}

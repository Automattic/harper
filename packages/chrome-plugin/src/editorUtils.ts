import { type Box, domRectToBox } from './Box';
import TextFieldRange from './TextFieldRange';

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

export function findChild(
	el: HTMLElement,
	predicate: (el: HTMLElement) => boolean,
): HTMLElement | null {
	const queue: HTMLElement[] = Array.from(el.children) as HTMLElement[];

	while (queue.length > 0) {
		const node = queue.shift() as HTMLElement;

		if (predicate(node)) {
			return node;
		}

		queue.push(...(Array.from(node.children) as HTMLElement[]));
	}

	return null;
}

/** Determines if a given node is a child of a P2 editor instance.
 * If so, returns the root node of that instance. */
export function getP2Root(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('p2-editor'));
}

/** Determines if a given node is a child of a Gutenberg editor instance.
 * If so, returns the root node of that instance. */
export function getGutenbergRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) =>
		node.classList.contains('block-editor-block-canvas'),
	);
}

/** Determines if a given node is a child of a Lexical editor instance.
 * If so, returns the root node of that instance. */
export function getLexicalRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(
		el,
		(node: HTMLElement) => node.getAttribute('data-lexical-editor') == 'true',
	);
}

export function getLexicalEditable(el: HTMLElement): HTMLElement | null {
	const lexical = getLexicalRoot(el);

	if (lexical == null) {
		return null;
	}

	return findChild(lexical, (node: HTMLElement) => node.getAttribute('contenteditable') == 'true');
}

/** Determines if a given node is a child of a Slate.js editor instance.
 * If so, returns the root node of that instance. */
export function getSlateRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.getAttribute('data-slate-editor') == 'true');
}

/** Determines if a given node is a child of a Draft.js editor instance.
 * If so, returns the root node of that instance. */
export function getDraftRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('DraftEditor-root'));
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

/** Determines if a given node is a child of a CodeMirror editor instance.
 * If so, returns the root node of that instance. */
export function getCMRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('cm-editor'));
}

/** Determines if a given node is a child of a ProseMirror editor instance.
 * If so, returns the root node of that instance. */
export function getPMRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('ProseMirror'));
}

export function getCaretPosition(): Box | null {
	const active = document.activeElement;

	if (
		active instanceof HTMLTextAreaElement ||
		(active instanceof HTMLInputElement && active.type === 'text')
	) {
		if (
			active.selectionStart == null ||
			active.selectionEnd == null ||
			active.selectionStart !== active.selectionEnd
		) {
			return null;
		}

		const offset = active.selectionStart;
		const tfRange = new TextFieldRange(active, offset, offset);
		const rects = tfRange.getClientRects();
		tfRange.detach();

		return rects.length ? domRectToBox(rects[0]) : null;
	}

	const selection = window.getSelection();
	if (!selection || selection.rangeCount === 0) return null;

	const range = selection.getRangeAt(0);
	if (!range.collapsed) return null;

	return domRectToBox(range.getBoundingClientRect());
}

import { type Box, domRectToBox } from './Box';
import TextFieldRange from './TextFieldRange';

export function findAncestor(
	el: HTMLElement,
	predicate: (el: HTMLElement) => boolean,
): HTMLElement | null {
	let current: HTMLElement | null = el;
	while (current != null) {
		if (predicate(current)) return current;
		current = current.parentElement;
	}
	return null;
}

export function getGhostRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.closest('article, main, section') != null);
}

export function getDraftRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) =>
		node.classList.contains('public-DraftEditor-content'),
	);
}

export function getPMRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('ProseMirror'));
}

export function getCMRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('cm-editor'));
}

export function getNotionRoot(): HTMLElement | null {
	return document.getElementById('notion-app');
}

export function getSlateRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.getAttribute('data-slate-editor') === 'true');
}

export function getLexicalRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(
		el,
		(node: HTMLElement) => node.getAttribute('data-lexical-editor') === 'true',
	);
}

export function getLexicalEditable(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.getAttribute('contenteditable') === 'true');
}

export function getMediumRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(
		el,
		(node: HTMLElement) => node.nodeName == 'MAIN' && location.hostname == 'medium.com',
	);
}

export function getShredditComposerRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.nodeName == 'SHREDDIT-COMPOSER');
}

export function getQuillJsRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.classList.contains('ql-container'));
}

export function getP2Root(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.id === 'p2' || node.classList.contains('p2'));
}

export function getGutenbergRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(
		el,
		(node: HTMLElement) => node.id === 'editor' || node.classList.contains('editor-styles-wrapper'),
	);
}

export function getTrixRoot(el: HTMLElement): HTMLElement | null {
	return findAncestor(el, (node: HTMLElement) => node.nodeName == 'TRIX-EDITOR');
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

export function isFormEl(el: any): el is HTMLInputElement | HTMLTextAreaElement {
	return el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement;
}

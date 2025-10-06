import { type Box, domRectToBox } from './Box';
import type SourceElement from './SourceElement';
import TextFieldRange from './TextFieldRange';

export function findAncestor(
	el: SourceElement,
	predicate: (el: SourceElement) => boolean,
): SourceElement | null {
	let current: SourceElement | null = el;
	while (current != null) {
		if (predicate(current)) return current;
		current = current.parentElement;
	}
	return null;
}

export function getGhostRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.closest('article, main, section') != null,
	);
}

export function getDraftRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) =>
			!isTextNode(node) && node.classList.contains('public-DraftEditor-content'),
	);
}

export function getPMRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.classList.contains('ProseMirror'),
	);
}

export function getCMRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.classList.contains('cm-editor'),
	);
}

export function getNotionRoot(): SourceElement | null {
	return document.getElementById('notion-app');
}

export function getSlateRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.getAttribute('data-slate-editor') === 'true',
	);
}

export function getLexicalRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) =>
			!isTextNode(node) && node.getAttribute('data-lexical-editor') === 'true',
	);
}

export function getLexicalEditable(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.getAttribute('contenteditable') === 'true',
	);
}

export function getMediumRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => node.nodeName == 'MAIN' && location.hostname == 'medium.com',
	);
}

export function getShredditComposerRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.nodeName == 'SHREDDIT-COMPOSER',
	);
}

export function getQuillJsRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) => !isTextNode(node) && node.classList.contains('ql-container'),
	);
}

export function getP2Root(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) =>
			!isTextNode(node) && (node.id === 'p2' || node.classList.contains('p2')),
	);
}

export function getGutenbergRoot(el: SourceElement): SourceElement | null {
	return findAncestor(
		el,
		(node: SourceElement) =>
			!isTextNode(node) &&
			(node.id === 'editor' || node.classList.contains('editor-styles-wrapper')),
	);
}

export function getTrixRoot(el: SourceElement): SourceElement | null {
	return findAncestor(el, (node: SourceElement) => node.nodeName == 'TRIX-EDITOR');
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

export function isTextNode(el: SourceElement): el is Text {
	return el.nodeType === Node.TEXT_NODE;
}

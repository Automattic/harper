import type { Span } from 'harper.js';
import { isBoxInScreen } from './Box';

/**
 * Turn a `NodeList` into a normal JavaScript array.
 * @param collection
 */
export function extractFromHTMLCollection(collection: HTMLCollection): Element[] {
	const elements: Element[] = [];
	for (let i = 0; i < collection.length; i++) {
		const el = collection.item(i);
		if (el) elements.push(el);
	}
	return elements;
}

/**
 * Turn a `NodeList` into a normal JavaScript array.
 * @param list
 */
export function extractFromNodeList<T extends Node>(list: NodeListOf<T>): T[] {
	const elements: T[] = [];

	for (let i = 0; i < list.length; i++) {
		const item = list[i];
		elements.push(item);
	}

	return elements;
}

export function getNodesFromQuerySelector(element: Element, query: string) {
	return extractFromNodeList(element.querySelectorAll(query));
}

/** Get a node's closest ancestor that has `display: block`. */
export function getClosestBlockAncestor(leaf: Node, root: Element): Element | null {
	let current: Node | null = leaf;

	while (current) {
		if (current instanceof Element) {
			if (getComputedStyle(current).display === 'block') {
				return current;
			}

			if (current === root) {
				break;
			}
		}

		current = current.parentNode;
	}

	return null;
}

/**
 * Flatten a provided node, and its children into a single array.
 * @param node
 */
export function leafNodes(node: Node): Node[] {
	const out: Node[] = [];

	const children = extractFromNodeList(node.childNodes);

	if (children.length === 0) {
		return [node];
	}

	for (const child of children) {
		const sub = leafNodes(child);
		sub.forEach((v) => {
			out.push(v);
		});
	}

	return out;
}

/**
 * Given an element and a Span of text inside it, compute the Range that represents the region of the DOM represented.
 * Accounts for `<br>` elements which `innerText` converts to newlines.
 * @param target
 * @param span
 */
export function getRangeForTextSpan(target: Element, span: Span): Range | null {
	const children = leafNodes(target);

	const range = target.ownerDocument.createRange();
	let traversed = 0;

	let startFound = false;

	for (let i = 0; i < children.length; i++) {
		const child = children[i] as HTMLElement;

		if (child.nodeName === 'BR') {
			traversed += 1;
			continue;
		}

		const childText = child.textContent ?? '';

		if (traversed + childText.length > span.start && !startFound) {
			range.setStart(child, span.start - traversed);
			startFound = true;
		}

		if (startFound && traversed + childText.length >= span.end) {
			range.setEnd(child, span.end - traversed);
			return range;
		}

		traversed += childText?.length ?? 0;
	}

	return null;
}

function getPointInTextSpan(
	target: Element,
	offset: number,
): { node: Node; offset: number } | null {
	const children = leafNodes(target);
	let traversed = 0;
	let lastTextNode: Text | null = null;

	for (const child of children) {
		if (child.nodeName === 'BR') {
			if (traversed === offset) {
				return { node: child.parentElement ?? target, offset: 0 };
			}
			traversed += 1;
			continue;
		}

		const childText = child.textContent ?? '';
		if (child.nodeType === Node.TEXT_NODE) {
			lastTextNode = child as Text;
		}

		if (traversed + childText.length >= offset) {
			return { node: child, offset: offset - traversed };
		}

		traversed += childText.length;
	}

	if (lastTextNode != null) {
		return { node: lastTextNode, offset: lastTextNode.textContent?.length ?? 0 };
	}

	return { node: target, offset: target.childNodes.length };
}

function getCodeMirrorLineAndColumn(
	lines: HTMLElement[],
	offset: number,
): { line: HTMLElement; column: number } | null {
	let remaining = offset;

	for (const line of lines) {
		const lineLength = line.textContent?.length ?? 0;

		if (remaining <= lineLength) {
			return { line, column: remaining };
		}

		remaining -= lineLength + 1;
	}

	const lastLine = lines[lines.length - 1];
	if (lastLine == null) {
		return null;
	}

	return { line: lastLine, column: lastLine.textContent?.length ?? 0 };
}

/**
 * Given a CodeMirror `.cm-content` element and a span in the text produced by joining
 * `.cm-line` contents with newlines, compute the equivalent DOM Range.
 */
export function getRangeForCodeMirrorTextSpan(target: Element, span: Span): Range | null {
	const lines = Array.from(target.querySelectorAll<HTMLElement>('.cm-line'));
	const start = getCodeMirrorLineAndColumn(lines, span.start);
	const end = getCodeMirrorLineAndColumn(lines, span.end);

	if (start == null || end == null) {
		return null;
	}

	const startPoint = getPointInTextSpan(start.line, start.column);
	const endPoint = getPointInTextSpan(end.line, end.column);

	if (startPoint == null || endPoint == null) {
		return null;
	}

	const range = target.ownerDocument.createRange();
	range.setStart(startPoint.node, startPoint.offset);
	range.setEnd(endPoint.node, endPoint.offset);
	return range;
}

const sharedRange: Range | null = typeof document !== 'undefined' ? document.createRange() : null;

/** Check if a node represents a heading (native heading tags or role="heading"). */
export function isHeading(node: Node): boolean {
	if (!(node instanceof Element)) return false;

	const tag = node.tagName.toLowerCase();
	if (/^h[1-6]$/.test(tag)) return true;

	const role = node.getAttribute('role');
	return role?.toLowerCase() === 'heading';
}

/** Check if an element is visible to the user.
 *
 * It is coarse and meant for performance improvements, not precision.*/
export function isVisible(node: Node): boolean {
	try {
		if (!node || !(node as any).ownerDocument) return false;

		if (node instanceof Element) {
			if (!node.isConnected) return false;

			// Google Docs integration uses an off-screen bridge element that is intentionally
			// hidden from users. Treat it as visible when its editor container is on-screen.
			if (node.getAttribute('data-harper-google-docs-target') === 'true') {
				const editor = node.closest('.kix-appview-editor') as HTMLElement | null;
				if (!editor) return false;
				return isBoxInScreen(editor.getBoundingClientRect());
			}

			const rect = node.getBoundingClientRect();
			if (!isBoxInScreen(rect)) return false;
			const cv = (node as any).checkVisibility;
			if (typeof cv === 'function') return cv.call(node);
			const cs = getComputedStyle(node);
			if (cs.display === 'none' || cs.visibility === 'hidden' || cs.opacity === '0') return false;
			return true;
		}

		if (!sharedRange) return false;
		const parent = (node as any).parentElement as Element | null;
		if (parent && !parent.isConnected) return false;
		sharedRange.selectNode(node);
		const rect = sharedRange.getBoundingClientRect();
		return isBoxInScreen(rect);
	} catch {
		return false;
	}
}

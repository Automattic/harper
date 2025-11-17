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
 * @param target
 * @param span
 */
export function getRangeForTextSpan(target: Element, span: Span): Range | null {
	const children = leafNodes(target);

	const range = document.createRange();
	let traversed = 0;

	let startFound = false;

	for (let i = 0; i < children.length; i++) {
		const child = children[i] as HTMLElement;
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

const sharedRange: Range | null = typeof document !== 'undefined' ? document.createRange() : null;

/** Check if an element is visible to the user.
 *
 * It is coarse and meant for performance improvements, not precision.*/
export function isVisible(node: Node): boolean {
	try {
		if (!node || !(node as any).ownerDocument) return false;

		if (node instanceof Element) {
			if (!node.isConnected) return false;
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

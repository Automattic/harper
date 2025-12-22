import type { Span } from 'harper.js';

/**
 * Turn a `NodeList` into a normal JavaScript array.
 * @param collection
 */
export function extractFromHTMLCollection(collection: HTMLCollection): Element[] {
	const elements: Element[] = [];

	for (const el of collection) {
		elements.push(el);
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

/**
 * Locate the rich text containers inside a given element.
 * Notice: this function may return the provided element.
 * @param target
 */
export function getRichTextContainers(target: Element): Element[] {
	const elms: Element[] = [];

	if (target.classList.contains('rich-text')) {
		elms.push(target);
	}

	elms.push(...extractFromHTMLCollection(target.getElementsByClassName('rich-text')));

	return elms;
}

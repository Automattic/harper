import { type IgnorableLintBox, type LintBox, domRectToBox, isBottomEdgeInBox } from './Box';
import ProtocolClient from './ProtocolClient';
import TextFieldRange from './TextFieldRange';
import { getRangeForTextSpan } from './domUtils';
import { type UnpackedLint, type UnpackedSuggestion, applySuggestion } from './unpackLint';

function isFormEl(el: HTMLElement): el is HTMLTextAreaElement | HTMLInputElement {
	switch (el.tagName) {
		case 'TEXTAREA':
		case 'INPUT':
			return true;
		default:
			return false;
	}
}

export default function computeLintBoxes(el: HTMLElement, lint: UnpackedLint): IgnorableLintBox[] {
	let range: Range | TextFieldRange;
	let text: string | null = null;

	if (isFormEl(el)) {
		range = new TextFieldRange(el, lint.span.start, lint.span.end);
		text = el.value;
	} else {
		range = getRangeForTextSpan(el, lint.span);
	}

	const targetRects = range.getClientRects();
	const elBox = domRectToBox(range.getBoundingClientRect());
	range.detach();

	const boxes: IgnorableLintBox[] = [];

	let source: HTMLElement | null = null;

	if (el.tagName == undefined) {
		source = el.parentElement;
	} else {
		source = el;
	}

	if (source == null) {
		return [];
	}

	for (const targetRect of targetRects) {
		if (!isBottomEdgeInBox(targetRect, elBox)) {
			continue;
		}

		boxes.push({
			x: targetRect.x,
			y: targetRect.y,
			width: targetRect.width,
			height: targetRect.height,
			lint,
			source,
			applySuggestion: (sug: UnpackedSuggestion) => {
				replaceValue(el, applySuggestion(el.value ?? el.textContent, lint.span, sug));
			},
			ignoreLint: () => ProtocolClient.ignoreHash(lint.context_hash),
		});
	}

	return boxes;
}

function replaceValue(el: HTMLElement, value: string) {
	if (isFormEl(el)) {
		el.value = value;
	} else {
		el.textContent = value;

		el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
		el.dispatchEvent(new InputEvent('input', { bubbles: true }));
		console.log(document.contains(el));
	}

	el.dispatchEvent(new Event('change', { bubbles: true }));
}

import type { Span } from 'harper.js';
import { domRectToBox, type IgnorableLintBox, isBottomEdgeInBox, shrinkBoxToFit } from './Box';
import { getRangeForTextSpan } from './domUtils';
import { getLexicalEditable, getSlateRoot } from './editorUtils';
import ProtocolClient from './ProtocolClient';
import TextFieldRange from './TextFieldRange';
import { applySuggestion, type UnpackedLint, type UnpackedSuggestion } from './unpackLint';

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
	try {
		let range: Range | TextFieldRange | null = null;
		let text: string | null = null;

		if (isFormEl(el)) {
			range = new TextFieldRange(el, lint.span.start, lint.span.end);
			text = el.value;
		} else {
			range = getRangeForTextSpan(el, lint.span as Span);
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

			const shrunkBox = shrinkBoxToFit(targetRect, elBox);

			boxes.push({
				x: shrunkBox.x,
				y: shrunkBox.y,
				width: shrunkBox.width,
				height: shrunkBox.height,
				lint,
				source,
				applySuggestion: (sug: UnpackedSuggestion) => {
					replaceValue(el, applySuggestion(el.value ?? el.textContent, lint.span, sug));
				},
				ignoreLint: () => ProtocolClient.ignoreHash(lint.context_hash),
			});
		}
		return boxes;
	} catch (e) {
		// If there's an error, it's likely because the element no longer exists
		return [];
	}
}

function replaceValue(el: HTMLElement, value: string) {
	const slateRoot = getSlateRoot(el);
	const lexicalRoot = getLexicalEditable(el);

	if (isFormEl(el)) {
		el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
		el.value = value;
		el.dispatchEvent(new InputEvent('input', { bubbles: true }));
	} else if (slateRoot != null || lexicalRoot != null) {
		replaceValueSpecial(el, value);
	} else {
		el.textContent = value;

		el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
		el.dispatchEvent(new InputEvent('input', { bubbles: true }));
	}

	el.dispatchEvent(new Event('change', { bubbles: true }));
}

/** Replace the content of a special editor node. */
function replaceValueSpecial(el: HTMLElement, value: string) {
	specialSelectAllText(el);
	specialInsertText(el, value);
}

function specialSelectAllText(target: Node): Range {
	const range = target.ownerDocument!.createRange();
	if (target.nodeType === Node.TEXT_NODE) {
		const len = (target as Text).data.length;
		range.setStart(target, 0);
		range.setEnd(target, len);
	} else {
		range.selectNodeContents(target);
	}
	const sel = target.ownerDocument!.defaultView!.getSelection();
	sel?.removeAllRanges();
	sel?.addRange(range);
	return range;
}

function specialInsertText(el: HTMLElement, raw: string): void {
	const inputType = 'insertText';

	const evInit: InputEventInit = {
		bubbles: true,
		cancelable: true,
		inputType,
		data: raw,
	};

	if ('StaticRange' in self && 'getTargetRanges' in InputEvent.prototype) {
		const sel = el.ownerDocument.defaultView!.getSelection();
		if (sel?.rangeCount) evInit.targetRanges = [new StaticRange(sel.getRangeAt(0))];
	}

	const beforeEvt = new InputEvent('beforeinput', evInit);
	const biSuccess: boolean = el.dispatchEvent(beforeEvt);

	const textEvt = new InputEvent('textInput', evInit);
	const teSuccess = el.dispatchEvent(textEvt);

	if (biSuccess && teSuccess) {
		el.ownerDocument.execCommand(inputType, false, raw);
	}
}

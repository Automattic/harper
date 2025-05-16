import { type IgnorableLintBox, type LintBox, domRectToBox, isBottomEdgeInBox } from './Box';
import ProtocolClient from './ProtocolClient';
import TextFieldRange from './TextFieldRange';
import { getRangeForTextSpan } from './domUtils';
import { getSlateRoot } from './editorUtils';
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
	const slateRoot = getSlateRoot(el);

	if (isFormEl(el)) {
		el.value = value;
	} else if (slateRoot != null) {
		replaceValueSlate(el, value);
	} else {
		el.textContent = value;

		el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
		el.dispatchEvent(new InputEvent('input', { bubbles: true }));
	}

	el.dispatchEvent(new Event('change', { bubbles: true }));
}

/** Replace the content of a Slate editor node. */
function replaceValueSlate(el: HTMLElement, value: string) {
	slateSelectAllText(el);
	slateInsertText(el, value);
}

function slateSelectAllText(target: Node): Range {
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

function slateInsertText(el: HTMLElement, raw: string): void {
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

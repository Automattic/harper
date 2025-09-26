import type { Span } from 'harper.js';
import { domRectToBox, type IgnorableLintBox, isBottomEdgeInBox, shrinkBoxToFit } from './Box';
import { getRangeForTextSpan } from './domUtils';
import { getLexicalEditable, getSlateRoot } from './editorUtils';
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

export default function computeLintBoxes(
	el: HTMLElement,
	lint: UnpackedLint,
	opts: { ignoreLint?: (hash: string) => Promise<void> },
): IgnorableLintBox[] {
	try {
		let range: Range | TextFieldRange | null = null;

		if (isFormEl(el)) {
			range = new TextFieldRange(el, lint.span.start, lint.span.end);
		} else {
			range = getRangeForTextSpan(el, lint.span as Span);
		}

		if (!range) {
			return [];
		}

		const targetRects = Array.from(
			(range as Range).getClientRects ? (range as Range).getClientRects() : [],
		);
		const elBox = domRectToBox((range as Range).getBoundingClientRect());
		(range as any).detach?.();

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

		for (const targetRect of targetRects as DOMRect[]) {
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
				range: range instanceof Range ? range : undefined,
				applySuggestion: (sug: UnpackedSuggestion) => {
					const current = isFormEl(el)
						? (el as HTMLInputElement | HTMLTextAreaElement).value
						: (el.textContent ?? '');
					replaceValue(el, applySuggestion(current, lint.span, sug));
				},
				ignoreLint: opts.ignoreLint ? () => opts.ignoreLint!(lint.context_hash) : undefined,
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
		(el as any).value = value;
		el.dispatchEvent(new InputEvent('input', { bubbles: true }));
	} else if (slateRoot != null || lexicalRoot != null) {
		replaceValueSpecial(el, value);
	} else {
		(el as any).textContent = value;

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

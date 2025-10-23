import type { Span } from 'harper.js';
import { domRectToBox, type IgnorableLintBox, isBottomEdgeInBox, shrinkBoxToFit } from './Box';
import { getRangeForTextSpan } from './domUtils';
import { getLexicalRoot, getSlateRoot } from './editorUtils';
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
	rule: string,
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
				rule,
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
	if (isFormEl(el)) {
		replaceFormElementValue(el as HTMLTextAreaElement | HTMLInputElement, value);
	} else if (getLexicalRoot(el) != null) {
		replaceRichTextValue(el, value, { mode: 'lexical' });
	} else if (getSlateRoot(el) != null) {
		replaceRichTextValue(el, value, { mode: 'slate' });
	} else {
		replaceGenericContentEditable(el, value);
	}

	el.dispatchEvent(new Event('change', { bubbles: true }));
}

function replaceFormElementValue(el: HTMLTextAreaElement | HTMLInputElement, value: string) {
	el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
	el.value = value;
	el.dispatchEvent(new InputEvent('input', { bubbles: true }));
}

function replaceRichTextValue(el: HTMLElement, value: string, opts: { mode: 'lexical' | 'slate' }) {
	specialSelectAllText(el);
	specialInsertText(el, value, opts);
}

function replaceGenericContentEditable(el: HTMLElement, value: string) {
	el.textContent = value;
	el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
	el.dispatchEvent(new InputEvent('input', { bubbles: true }));
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

function getEditorText(el: HTMLElement): string {
	const text = el.textContent ?? '';
	return normalizeEditorText(text);
}

function normalizeEditorText(text: string): string {
	return text.replace(/\u200b/g, '').replace(/[\n\r]+$/g, '');
}

function specialInsertText(
	el: HTMLElement,
	raw: string,
	opts: { mode: 'lexical' | 'slate' },
): void {
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
	const biSuccess = el.dispatchEvent(beforeEvt);
	if (getEditorText(el) === raw) {
		return;
	}

	const textEvt = new InputEvent('textInput', evInit);
	const teSuccess = el.dispatchEvent(textEvt);
	if (getEditorText(el) === raw) {
		return;
	}

	const finalize = () => {
		if (getEditorText(el) !== raw) {
			el.textContent = raw;
		}
	};

	const shouldRunExecCommand = opts.mode !== 'lexical' && (!biSuccess || !teSuccess);
	if (shouldRunExecCommand) {
		el.ownerDocument.execCommand(inputType, false, raw);
		finalize();
		return;
	}

	setTimeout(finalize, 0);
}

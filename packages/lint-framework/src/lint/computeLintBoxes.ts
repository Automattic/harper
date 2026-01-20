import type { Span } from 'harper.js';
import { domRectToBox, type IgnorableLintBox, isBottomEdgeInBox, shrinkBoxToFit } from './Box';
import { getRangeForTextSpan } from './domUtils';
import {
	getCkEditorRoot,
	getDraftRoot,
	getLexicalRoot,
	getSlateRoot,
	isFormEl,
} from './editorUtils';
import TextFieldRange from './TextFieldRange';
import { applySuggestion, type UnpackedLint, type UnpackedSuggestion } from './unpackLint';

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
					const newValue = applySuggestion(current, lint.span, sug);
					replaceValue(el, newValue, lint.span, sug.replacement_text);
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

function replaceValue(
	el: HTMLElement,
	value: string,
	span?: { start: number; end: number },
	replacementText?: string,
) {
	if (isFormEl(el)) {
		replaceFormElementValue(el as HTMLTextAreaElement | HTMLInputElement, value);
	} else if (getLexicalRoot(el) != null) {
		replaceLexicalValue(el, value);
	} else if (getSlateRoot(el) != null && span && replacementText !== undefined) {
		replaceSlateValue(el, span, replacementText);
	} else if (getCkEditorRoot(el) != null && span && replacementText !== undefined) {
		replaceSlateValue(el, span, replacementText);
	} else if (getDraftRoot(el) != null && span && replacementText !== undefined) {
		replaceDraftJsValue(el, span, replacementText);
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

function replaceLexicalValue(el: HTMLElement, value: string) {
	// Select all text
	const range = el.ownerDocument!.createRange();
	if (el.nodeType === Node.TEXT_NODE) {
		const len = (el as unknown as Text).data.length;
		range.setStart(el, 0);
		range.setEnd(el, len);
	} else {
		range.selectNodeContents(el);
	}
	const sel = el.ownerDocument!.defaultView!.getSelection();
	sel?.removeAllRanges();
	sel?.addRange(range);

	// Insert new text
	const evInit: InputEventInit = {
		bubbles: true,
		cancelable: true,
		inputType: 'insertText',
		data: value,
	};

	if ('StaticRange' in self && 'getTargetRanges' in InputEvent.prototype) {
		if (sel?.rangeCount) evInit.targetRanges = [new StaticRange(sel.getRangeAt(0))];
	}

	el.dispatchEvent(new InputEvent('beforeinput', evInit));
	if (getEditorText(el) === value) {
		return;
	}

	el.dispatchEvent(new InputEvent('textInput', evInit));
	if (getEditorText(el) === value) {
		return;
	}

	setTimeout(() => {
		if (getEditorText(el) !== value) {
			el.textContent = value;
		}
	}, 0);
}

function selectSpanInEditor(el: HTMLElement, span: { start: number; end: number }) {
	const doc = el.ownerDocument;
	const sel = doc.defaultView?.getSelection();

	if (!sel) {
		return null;
	}

	el.focus();

	const range = getRangeForTextSpan(el, span as Span);
	if (!range) {
		return null;
	}

	sel.removeAllRanges();
	sel.addRange(range);

	return { doc, range };
}

function replaceDraftJsValue(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	const setup = selectSpanInEditor(el, span);
	if (!setup) return;

	setup.doc.execCommand('insertText', false, replacementText);
}

function replaceSlateValue(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	const setup = selectSpanInEditor(el, span);
	if (!setup) return;

	const { doc, range } = setup;

	const evInit: InputEventInit = {
		bubbles: true,
		cancelable: true,
		inputType: 'insertText',
		data: replacementText,
	};

	if ('StaticRange' in self) {
		evInit.targetRanges = [new StaticRange(range)];
	}

	const beforeEvt = new InputEvent('beforeinput', evInit);
	el.dispatchEvent(beforeEvt);

	if (!beforeEvt.defaultPrevented) {
		doc.execCommand('insertText', false, replacementText);
	}
}

function replaceGenericContentEditable(el: HTMLElement, value: string) {
	el.textContent = value;
	el.dispatchEvent(new InputEvent('beforeinput', { bubbles: true, data: value }));
	el.dispatchEvent(new InputEvent('input', { bubbles: true }));
}

function getEditorText(el: HTMLElement): string {
	const text = el.textContent ?? '';
	return normalizeEditorText(text);
}

function normalizeEditorText(text: string): string {
	return text.replace(/\u200b/g, '').replace(/[\n\r]+$/g, '');
}

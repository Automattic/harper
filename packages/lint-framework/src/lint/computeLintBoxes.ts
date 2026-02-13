import { type Span, SuggestionKind } from 'harper.js';
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
import {
	applySuggestion,
	type UnpackedLint,
	type UnpackedSpan,
	type UnpackedSuggestion,
} from './unpackLint';

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
					replaceValue(el, lint.span, suggestionToReplacementText(sug, lint.span, current));
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

/** Transform an arbitrary suggestion to the equivalent replacement text. */
function suggestionToReplacementText(
	sug: UnpackedSuggestion,
	span: UnpackedSpan,
	source: string,
): string {
	switch (sug.kind) {
		case SuggestionKind.Replace:
			return sug.replacement_text;
		case SuggestionKind.Remove:
			return '';
		case SuggestionKind.InsertAfter:
			return source.slice(span.start, span.end) + sug.replacement_text;
	}
}

function replaceValue(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	if (isFormEl(el)) {
		replaceFormElementValue(el as HTMLTextAreaElement | HTMLInputElement, span, replacementText);
	} else if (getLexicalRoot(el) != null) {
		replaceLexicalValue(el, span, replacementText);
	} else if (getDraftRoot(el) != null) {
		replaceDraftValue(el, span, replacementText);
	} else if (getSlateRoot(el) != null || getCkEditorRoot(el) != null) {
		replaceRichTextEditorValue(el, span, replacementText);
	} else {
		replaceGenericContentEditable(el, span, replacementText);
	}

	el.dispatchEvent(new Event('change', { bubbles: true }));
}

function replaceFormElementValue(
	el: HTMLTextAreaElement | HTMLInputElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	el.focus();
	el.setSelectionRange(span.start, span.end);
	document.execCommand('insertText', false, replacementText);
}

function replaceLexicalValue(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	const setup = selectSpanInEditor(el, span);
	if (!setup) return;

	const { doc, sel, range } = setup;

	// Direct DOM replacement
	replaceTextInRange(doc, sel, range, replacementText);

	// Notify
	el.dispatchEvent(new InputEvent('input', { bubbles: true, cancelable: false }));
}

function replaceDraftValue(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	const setup = selectSpanInEditor(el, span);
	if (!setup) return;

	const { doc, sel, range } = setup;

	setTimeout(() => {
		const beforeEvt = new InputEvent('beforeinput', {
			bubbles: true,
			cancelable: true,
			inputType: 'insertText',
			data: replacementText,
		});
		el.dispatchEvent(beforeEvt);

		if (!beforeEvt.defaultPrevented) {
			replaceTextInRange(doc, sel, range, replacementText);
		}

		el.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText' }));
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

	return { doc, sel, range };
}

function replaceRichTextEditorValue(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	const setup = selectSpanInEditor(el, span);
	if (!setup) return;

	const { doc, sel, range } = setup;

	const evInit: InputEventInit = {
		bubbles: true,
		cancelable: true,
		inputType: 'insertReplacementText',
		data: replacementText,
	};

	if ('StaticRange' in self) {
		evInit.targetRanges = [new StaticRange(range)];
	}

	const beforeEvt = new InputEvent('beforeinput', evInit);
	el.dispatchEvent(beforeEvt);

	if (!beforeEvt.defaultPrevented) {
		replaceTextInRange(doc, sel, range, replacementText);
		el.dispatchEvent(new InputEvent('input', { bubbles: true, cancelable: false }));
	}
}

function replaceTextInRange(doc: Document, sel: Selection, range: Range, replacementText: string) {
	const startContainer = range.startContainer;
	const endContainer = range.endContainer;

	if (startContainer === endContainer && startContainer.nodeType === Node.TEXT_NODE) {
		const textNode = startContainer as Text;
		const startOffset = range.startOffset;
		const endOffset = range.endOffset;

		const oldText = textNode.textContent || '';
		const newText =
			oldText.substring(0, startOffset) + replacementText + oldText.substring(endOffset);

		textNode.textContent = newText;

		// Set cursor after replacement
		const newRange = doc.createRange();
		const cursorPosition = startOffset + replacementText.length;
		newRange.setStart(textNode, cursorPosition);
		newRange.setEnd(textNode, cursorPosition);
		sel.removeAllRanges();
		sel.addRange(newRange);
	} else {
		// Multi node range fallback
		range.deleteContents();
		const textNode = doc.createTextNode(replacementText);
		range.insertNode(textNode);

		const newRange = doc.createRange();
		newRange.setStartAfter(textNode);
		newRange.setEndAfter(textNode);
		sel.removeAllRanges();
		sel.addRange(newRange);
	}
}

function replaceGenericContentEditable(
	el: HTMLElement,
	span: { start: number; end: number },
	replacementText: string,
) {
	if (span && replacementText !== undefined) {
		const setup = selectSpanInEditor(el, span);
		if (setup) {
			const { doc, sel, range } = setup;
			replaceTextInRange(doc, sel, range, replacementText);
			el.dispatchEvent(new InputEvent('input', { bubbles: true, cancelable: false }));
			return;
		}
	}

	// Fallback: replace entire content
	el.textContent = applySuggestion(el.textContent, span, {
		kind: SuggestionKind.Replace,
		replacement_text: replacementText,
	});
	el.dispatchEvent(new InputEvent('input', { bubbles: true }));
}

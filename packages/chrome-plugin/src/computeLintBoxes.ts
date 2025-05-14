import { type LintBox, domRectToBox, isBottomEdgeInBox } from './Box';
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

export default function computeLintBoxes(el: HTMLElement, lint: UnpackedLint): LintBox[] {
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

	const boxes: LintBox[] = [];

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
		});
	}

	return boxes;
}

function replaceValue(el: HTMLElement, value: string) {
	selectAllText(el);
	orchestratedInsert(el, value);
}

function selectAllText(target: Node): Range {
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

async function orchestratedInsert(
	el: HTMLElement,
	raw: string,
	staticRange: StaticRange | null = null,
): Promise<void> {
	const pause = (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

	const chunks: string[] = [];
	raw.split('\n').forEach((seg, idx, arr) => {
		if (seg) chunks.push(seg);
		if (idx + 1 < arr.length) chunks.push('\n');
	});

	for (let i = 0; i < chunks.length; i++) {
		const fragment: string = chunks[i];
		const isBreak: boolean = fragment === '\n';
		const inputType: 'insertText' | 'insertLineBreak' = isBreak ? 'insertLineBreak' : 'insertText';

		const evInit: InputEventInit = {
			bubbles: true,
			cancelable: true,
			inputType,
			data: fragment,
		};

		if ('StaticRange' in self && 'getTargetRanges' in InputEvent.prototype) {
			if (staticRange && i === 0) {
				evInit.targetRanges = [new StaticRange(staticRange)];
			} else {
				const sel = el.ownerDocument.defaultView!.getSelection();
				if (sel?.rangeCount) evInit.targetRanges = [new StaticRange(sel.getRangeAt(0))];
			}
		}

		if (isBreak) {
			const kdInit: KeyboardEventInit = {
				bubbles: true,
				cancelable: true,
				key: 'Enter',
				code: 'Enter',
				keyCode: 13,
				which: 13,
				shiftKey: true,
			};
			const kd = new KeyboardEvent('keydown', kdInit);
			if (!el.dispatchEvent(kd)) {
				await pause(10);
				continue;
			}
			const kp = new KeyboardEvent('keypress', { ...kdInit, charCode: 13 });
			if (!el.dispatchEvent(kp)) {
				await pause(10);
				continue;
			}
		}

		const beforeEvt = new InputEvent('beforeinput', evInit);
		const proceed1: boolean = el.dispatchEvent(beforeEvt);

		const ua: string = navigator.userAgent;
		const chromiumFamily: boolean = ua.includes('Chrome/') || ua.includes('Chromium/');
		const notSafari: boolean = !ua.includes('Safari/') || chromiumFamily;

		let proceed2 = true;
		if ('TextEvent' in self && notSafari) {
			const textEvt = new InputEvent('textInput', evInit);
			proceed2 = el.dispatchEvent(textEvt);
		}

		if (proceed1 && proceed2) {
			el.ownerDocument.execCommand(inputType, false, fragment);
		}

		await pause(10);
	}
}

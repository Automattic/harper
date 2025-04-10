import { type Lint, LocalLinter, Span, type Suggestion, binaryInlined } from 'harper.js';
import type { Box, LintBox } from './Box';
import TextareaRange from './TextareaRange';
import { applySuggestion, type UnpackedLint, type UnpackedSpan, type UnpackedSuggestion } from './unpackLint';

export type EditContentCallback = (newContent: string) => void;

export default class Textbox {
	private targetElement: HTMLTextAreaElement;

	constructor(targetElement: HTMLTextAreaElement) {
		this.targetElement = targetElement;
	}

	public getTargetElement(): Element {
		return this.targetElement;
	}

	public getTextContent(): string {
		return this.targetElement.textContent ?? '';
	}

	public computeBoxes(lint: UnpackedLint): LintBox[] {
		if (this.targetElement.tagName != 'TEXTAREA') {
			return [];
		}

		const range = new TextareaRange(
			this.targetElement as HTMLTextAreaElement,
			lint.span.start,
			lint.span.end,
		);

		const targetRects = range.getClientRects();
		range.detach();

		const boxes: LintBox[] = [];

		for (const targetRect of targetRects) {
			boxes.push({
				x: targetRect.x,
				y: targetRect.y,
				width: targetRect.width,
				height: targetRect.height,
				lint,
				applySuggestion: (sug: UnpackedSuggestion) => {
					this.targetElement.value = applySuggestion(this.targetElement.value, lint.span, sug);
				},
			});
		}

		return boxes;
	}
}

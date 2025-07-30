import { binaryInlined, type Lint, LocalLinter, type Suggestion } from 'harper.js';
import type { LintBox } from './Box';
import DataBlock from './DataBlock';
import { getRangeForTextSpan } from './domUtils';

export type EditContentCallback = (newContent: string) => void;

/**
 * Represents a rich text element on-screen.
 * It can either be a child element of a `DataBlock` or be the `DataBlock` itself.
 */
export default class RichText {
	private targetElement: Element;
	private parent: DataBlock;
	private editContent: EditContentCallback;

	constructor(targetElement: Element, parent: DataBlock, editContent: EditContentCallback) {
		this.targetElement = targetElement;
		this.parent = parent;
		this.editContent = editContent;
	}

	public getTargetElement(): Element {
		return this.targetElement;
	}

	public getTextContent(): string {
		return this.targetElement.textContent ?? '';
	}

	public computeLintBox(lint: Lint): LintBox[] {
		const text = this.targetElement.textContent;
		const span = lint.span();
		const range = getRangeForTextSpan(this.targetElement, span);
		// Use a local linter because we won't be doing any expensive operations with it.
		const linter = new LocalLinter({ binary: binaryInlined });

		if (range === null || text === null) {
			console.log('Could not locate range.');
			return [];
		}

		const targetRects = range.getClientRects();
		const container = DataBlock.getContainer();
		const contRect = container.getBoundingClientRect();

		const boxes: LintBox[] = [];

		for (const targetRect of targetRects) {
			boxes.push({
				x: targetRect.x - contRect.x,
				y: targetRect.y - contRect.y,
				width: targetRect.width,
				height: targetRect.height,
				lint,
				applySuggestion: async (sug: Suggestion) => {
					const fixed = await linter.applySuggestion(text, lint, sug);

					this.editContent(fixed);
				},
			});
		}

		return boxes;
	}
}

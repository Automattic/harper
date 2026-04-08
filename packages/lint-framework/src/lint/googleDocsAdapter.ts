import { type Span, SuggestionKind } from 'harper.js';
import { domRectToBox, type IgnorableLintBox, isBottomEdgeInBox, shrinkBoxToFit } from './Box';
import type SourceElement from './SourceElement';
import {
	getGoogleDocsHighlightRects,
	isGoogleDocsTarget,
	replaceGoogleDocsValue,
	resolveGoogleDocsSpan,
} from './computeLintBoxes/googleDocsUtilities';
import type { UnpackedLint, UnpackedSpan, UnpackedSuggestion } from './unpackLint';

const GOOGLE_DOCS_EDITOR_SELECTOR = '.kix-appview-editor';
const GOOGLE_DOCS_SYNCING_ATTR = 'data-harper-gdocs-syncing';

/**
 * Returns Google Docs-specific lint boxes for the hidden bridge target.
 *
 * Non-Google Docs targets return `null` so callers can continue down the generic
 * lint-box pipeline without duplicating source checks.
 */
export function maybeComputeGoogleDocsLintBoxes(
	target: HTMLElement,
	lint: UnpackedLint,
	rule: string,
	opts: { ignoreLint?: (hash: string) => Promise<void> },
): IgnorableLintBox[] | null {
	if (!isGoogleDocsTarget(target)) {
		return null;
	}

	try {
		const editor = document.querySelector(GOOGLE_DOCS_EDITOR_SELECTOR) as HTMLElement | null;
		const source = target.textContent ?? '';
		const resolvedSpan = resolveGoogleDocsSpan(source, lint);

		if (!editor || resolvedSpan == null) {
			return [];
		}

		const targetRects = getGoogleDocsHighlightRects(target, resolvedSpan as Span);
		const editorBox = domRectToBox(editor.getBoundingClientRect());
		if (targetRects.length === 0) {
			return [];
		}

		const boxes: IgnorableLintBox[] = [];
		for (const targetRect of targetRects as DOMRect[]) {
			if (!isBottomEdgeInBox(targetRect, editorBox)) {
				continue;
			}

			const shrunkBox = shrinkBoxToFit(targetRect, editorBox);
			boxes.push({
				x: shrunkBox.x,
				y: shrunkBox.y,
				width: shrunkBox.width,
				height: shrunkBox.height,
				lint,
				source: editor,
				rule,
				applySuggestion: (suggestion: UnpackedSuggestion) => {
					const replacementText = suggestionToReplacementText(suggestion, resolvedSpan, source);
					replaceGoogleDocsValue(resolvedSpan, replacementText, source);
				},
				ignoreLint: opts.ignoreLint ? () => opts.ignoreLint!(lint.context_hash) : undefined,
			});
		}

		return boxes;
	} catch {
		return [];
	}
}

/**
 * Narrows a render or lint source to the live Google Docs editor container.
 */
export function isGoogleDocsSource(source: SourceElement): source is HTMLElement {
	return (
		source instanceof HTMLElement &&
		(source.classList.contains('kix-appview-editor') ||
			source.closest(GOOGLE_DOCS_EDITOR_SELECTOR) != null)
	);
}

/**
 * Reports whether Google Docs is in the middle of a bridge sync and highlights
 * should preserve their current rendered state.
 */
export function isGoogleDocsSourceSyncing(source: SourceElement): boolean {
	return isGoogleDocsSource(source) && source.getAttribute(GOOGLE_DOCS_SYNCING_ATTR) === 'true';
}

/**
 * Returns the element that should own rendered highlight boxes for a Google Docs source.
 */
export function getGoogleDocsRenderTarget(source: SourceElement): HTMLElement | null {
	return isGoogleDocsSource(source) ? source : null;
}

/**
 * Computes the absolute render offset for highlight boxes anchored to the
 * scrolling Google Docs editor surface.
 */
export function getGoogleDocsRenderOffset(source: SourceElement): { x: number; y: number } | null {
	if (!isGoogleDocsSource(source)) {
		return null;
	}

	const editorRect = source.getBoundingClientRect();

	return {
		x: editorRect.x - source.scrollLeft,
		y: editorRect.y - source.scrollTop,
	};
}

/**
 * Applies the host positioning required for Google Docs highlights.
 *
 * Returns `true` when the source was recognized as Google Docs and the caller
 * should skip its generic host styling path.
 */
export function applyGoogleDocsRenderHostStyle(host: HTMLElement, source: SourceElement): boolean {
	if (!isGoogleDocsSource(source)) {
		return false;
	}

	host.style.position = 'absolute';
	host.style.top = '0px';
	host.style.left = '0px';
	host.style.pointerEvents = 'none';
	host.style.width = '0px';
	host.style.height = '0px';
	host.style.contain = 'none';
	host.style.transform = 'none';
	host.style.zIndex = '2147483647';
	return true;
}

function suggestionToReplacementText(
	suggestion: UnpackedSuggestion,
	span: UnpackedSpan,
	source: string,
): string {
	switch (suggestion.kind) {
		case SuggestionKind.Replace:
			return suggestion.replacement_text;
		case SuggestionKind.Remove:
			return '';
		case SuggestionKind.InsertAfter:
			return source.slice(span.start, span.end) + suggestion.replacement_text;
	}
}

import { type Span, SuggestionKind } from 'harper.js';
import { domRectToBox, type IgnorableLintBox, isBottomEdgeInBox, shrinkBoxToFit } from './Box';
import {
	getGoogleDocsHighlightRects,
	isGoogleDocsTarget,
	replaceGoogleDocsValue,
	resolveGoogleDocsSpan,
} from './computeLintBoxes/googleDocsUtilities';
import type SourceElement from './SourceElement';
import type { UnpackedLint, UnpackedSpan, UnpackedSuggestion } from './unpackLint';

const GOOGLE_DOCS_EDITOR_SELECTOR = '.kix-appview-editor';
const GOOGLE_DOCS_SYNCING_ATTR = 'data-harper-gdocs-syncing';
// biome-ignore lint/suspicious/noControlCharactersInRegex: Must match the content script's native table projection.
const GOOGLE_DOCS_TABLE_SEPARATORS = /[\u0010\u0011\u0012\u001c]/gu;
// biome-ignore lint/suspicious/noControlCharactersInRegex: Corrections must not cross native structural boundaries.
const GOOGLE_DOCS_CONTROL_CHARACTERS = /[\u0000-\u001f]/u;

function resolveNativeSpan(
	target: HTMLElement,
	source: string,
	nativeSnapshot: string,
	span: UnpackedSpan,
): UnpackedSpan | null {
	const runs: unknown = JSON.parse(target.getAttribute('data-harper-gdocs-native-runs') ?? 'null');
	if (!Array.isArray(runs) || runs.length === 0) return null;
	const masked = nativeSnapshot.replace(GOOGLE_DOCS_TABLE_SEPARATORS, '\n');
	let projectionCursor = 0;
	let nativeCursor = 0;
	let resolved: UnpackedSpan | null = null;
	for (const run of runs) {
		if (!Array.isArray(run) || run.length !== 3 || !run.every(Number.isInteger)) return null;
		const [projectionStart, nativeStart, length] = run;
		if (nativeCursor > 0) {
			// Only the extra LF of a paragraph boundary may be outside a run.
			if (masked[nativeCursor - 1] !== '\n' || source[projectionCursor] !== '\n') return null;
			projectionCursor += 1;
		}
		if (
			projectionStart !== projectionCursor ||
			nativeStart !== nativeCursor ||
			length <= 0 ||
			nativeStart + length > nativeSnapshot.length ||
			source.slice(projectionStart, projectionStart + length) !==
				masked.slice(nativeStart, nativeStart + length)
		)
			return null;
		if (span.start >= projectionStart && span.end <= projectionStart + length) {
			resolved = {
				start: nativeStart + span.start - projectionStart,
				end: nativeStart + span.end - projectionStart,
			};
		}
		nativeCursor += length;
		projectionCursor += length;
	}
	return nativeCursor === nativeSnapshot.length && projectionCursor === source.length
		? resolved
		: null;
}

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
		const sourceMode = target.getAttribute('data-harper-gdocs-source');
		const nativeSnapshot = target.getAttribute('data-harper-gdocs-native-snapshot');
		const mapped =
			sourceMode === 'native' ||
			nativeSnapshot != null ||
			target.hasAttribute('data-harper-gdocs-native-runs') ||
			target.querySelector('[data-harper-gdocs-native-start]') != null;
		// Capture provenance when the box is made, not when its action is clicked.
		// An old lint must not inherit the current target's native snapshot.
		const resolvedSpan = mapped
			? nativeSnapshot != null
				? resolveNativeSpan(target, source, nativeSnapshot, lint.span)
				: null
			: resolveGoogleDocsSpan(source, lint);

		if (!editor || resolvedSpan == null) {
			return [];
		}
		if (
			mapped &&
			(sourceMode !== 'native' ||
				nativeSnapshot == null ||
				lint.source !== source ||
				!Number.isInteger(lint.span.start) ||
				!Number.isInteger(lint.span.end) ||
				lint.span.start < 0 ||
				lint.span.end <= lint.span.start ||
				!Number.isInteger(resolvedSpan.start) ||
				!Number.isInteger(resolvedSpan.end) ||
				resolvedSpan.start < 0 ||
				resolvedSpan.end <= resolvedSpan.start ||
				resolvedSpan.end > nativeSnapshot.length ||
				GOOGLE_DOCS_CONTROL_CHARACTERS.test(
					nativeSnapshot.slice(resolvedSpan.start, resolvedSpan.end),
				) ||
				nativeSnapshot.slice(resolvedSpan.start, resolvedSpan.end) !== lint.problem_text)
		) {
			return [];
		}

		const targetRects = getGoogleDocsHighlightRects(
			target,
			(mapped ? lint.span : resolvedSpan) as Span,
		);
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
				applySuggestion: async (suggestion: UnpackedSuggestion) => {
					const replacementSource = nativeSnapshot ?? source;
					const replacementText = suggestionToReplacementText(
						suggestion,
						resolvedSpan,
						replacementSource,
					);
					await replaceGoogleDocsValue(
						resolvedSpan,
						replacementText,
						replacementSource,
						mapped
							? { rangeSource: 'native-utf16-v1', nativeSnapshot: nativeSnapshot! }
							: undefined,
					);
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

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
// biome-ignore lint/suspicious/noControlCharactersInRegex: Match extraction's one-unit lint projection of native Shift+Enter.
const GOOGLE_DOCS_SOFT_BREAKS = /\u000b/gu;
// biome-ignore lint/suspicious/noControlCharactersInRegex: Corrections must not cross native structural boundaries.
const GOOGLE_DOCS_CONTROL_CHARACTERS = /[\u0000-\u001f]/u;

function splitsSurrogate(text: string, offset: number): boolean {
	return (
		offset > 0 &&
		offset < text.length &&
		text.charCodeAt(offset - 1) >= 0xd800 &&
		text.charCodeAt(offset - 1) <= 0xdbff &&
		text.charCodeAt(offset) >= 0xdc00 &&
		text.charCodeAt(offset) <= 0xdfff
	);
}

function resolveOrdinarySpan(
	map: unknown,
	sourceMode: string | null,
	source: string,
	native: string,
	span: UnpackedSpan,
): UnpackedSpan | null {
	if (map == null || typeof map !== 'object' || Array.isArray(map)) return null;
	const descriptor = map as {
		format?: unknown;
		source?: unknown;
		window?: unknown;
		pieces?: unknown;
		regions?: unknown;
	};
	if (
		descriptor.format !== 'ordinary-utf16-v1' ||
		descriptor.source !== sourceMode ||
		(sourceMode !== 'logical' && sourceMode !== 'rects')
	)
		return null;
	const window = descriptor.window;
	if (descriptor.regions !== undefined) {
		if (
			sourceMode !== 'rects' ||
			!Array.isArray(descriptor.regions) ||
			descriptor.regions.length === 0 ||
			!Array.isArray(window) ||
			window.length !== 2 ||
			window[0] !== 0 ||
			window[1] !== 0 ||
			!Array.isArray(descriptor.pieces) ||
			descriptor.pieces.length !== 0
		)
			return null;
		let previousEnd = 0;
		const nativeWindows: Array<[number, number]> = [];
		let resolved: UnpackedSpan | null = null;
		for (const region of descriptor.regions) {
			if (!Array.isArray(region) || region.length !== 4 || !region.every(Number.isInteger))
				return null;
			const [p0, p1, n0, n1] = region;
			if (
				p0 < previousEnd ||
				p1 <= p0 ||
				p1 > source.length ||
				nativeWindows.some(([start, end]) => start < n1 && end > n0)
			)
				return null;
			// Independent regions prove complete exact paragraphs, not arbitrary
			// omitted native gaps or a synthetic linguistic join between them.
			const localSource = source.slice(p0, p1);
			const valid = resolveOrdinarySpan(
				{
					format: descriptor.format,
					source: sourceMode,
					window: [n0, n1],
					pieces: [['copy', 0, p1 - p0, n0, n1]],
				},
				sourceMode,
				localSource,
				native,
				{ start: 0, end: localSource.length },
			);
			if (valid == null) return null;
			if (
				span.start >= p0 &&
				span.end <= p1 &&
				// A complete native paragraph proves diagnostic context only when
				// it is also isolated from unproven text in the lint projection.
				(p0 === 0 || /\n[^\S\n]*\n\s*$/u.test(source.slice(0, p0))) &&
				(p1 === source.length || /^\s*\n[^\S\n]*\n/u.test(source.slice(p1)))
			)
				resolved = { start: n0 + span.start - p0, end: n0 + span.end - p0 };
			previousEnd = p1;
			nativeWindows.push([n0, n1]);
		}
		return resolved;
	}
	if (
		!Array.isArray(window) ||
		window.length !== 2 ||
		!window.every(Number.isInteger) ||
		window[0] < 0 ||
		window[1] <= window[0] ||
		window[1] > native.length ||
		!Array.isArray(descriptor.pieces) ||
		descriptor.pieces.length === 0
	)
		return null;
	if (
		sourceMode === 'logical'
			? window[0] !== 0 || window[1] !== native.length
			: (window[0] > 0 && !/[\r\n]/u.test(native[window[0] - 1])) ||
				(window[1] < native.length && !/[\r\n]/u.test(native[window[1]]))
	)
		return null;
	let p = 0;
	let n = window[0];
	let resolved: UnpackedSpan | null = null;
	let copyStart: { projection: number; native: number } | null = null;
	for (const piece of descriptor.pieces) {
		if (!Array.isArray(piece) || piece.length !== 5 || !piece.slice(1).every(Number.isInteger))
			return null;
		const [kind, p0, p1, n0, n1] = piece;
		if (p0 !== p || n0 !== n || p1 < p0 || n1 < n0 || p1 > source.length || n1 > native.length)
			return null;
		const projected = source.slice(p0, p1);
		const original = native.slice(n0, n1);
		if (kind === 'copy') {
			if (p1 === p0 || projected !== original) return null;
			copyStart ??= { projection: p0, native: n0 };
			if (span.start >= copyStart.projection && span.end <= p1)
				resolved = {
					start: copyStart.native + span.start - copyStart.projection,
					end: copyStart.native + span.end - copyStart.projection,
				};
		} else if (kind === 'gap') {
			const normalized = original.replace(/\r\n?/gu, '\n');
			if (
				!/^\s*$/u.test(original) ||
				(/\n[^\S\n]*\n/u.test(normalized) && !/\n[^\S\n]*\n/u.test(projected)) ||
				!(
					projected === normalized ||
					(original === '' && projected === ' ') ||
					projected === normalized.replace(/\n+/gu, '\n') ||
					projected === normalized.replace(/\n+/gu, '\n\n')
				)
			)
				return null;
			// Exact native whitespace preserves the contiguous offset chain.
			// Synthetic or normalized gaps cannot authorize a spanning edit.
			if (projected !== original) copyStart = null;
		} else return null;
		p = p1;
		n = n1;
	}
	if (sourceMode === 'rects') {
		const comparable = native.replace(/\r\n?/gu, '\n');
		const windowText = native.slice(window[0], window[1]).replace(/\r\n?/gu, '\n');
		let matches = 0;
		let cursor = 0;
		while (cursor < comparable.length) {
			const start = comparable.indexOf(windowText, cursor);
			if (start < 0) break;
			// Embedded and overlapping occurrences are ambiguous too, including
			// when this window validates an independent paragraph region.
			matches += 1;
			if (matches > 1) return null;
			cursor = start + 1;
		}
		if (matches !== 1) return null;
	}
	return p === source.length && n === window[1] ? resolved : null;
}

function resolveNativeSpan(
	target: HTMLElement,
	source: string,
	nativeSnapshot: string,
	span: UnpackedSpan,
): UnpackedSpan | null {
	const runs: unknown = JSON.parse(target.getAttribute('data-harper-gdocs-native-runs') ?? 'null');
	if (target.getAttribute('data-harper-gdocs-source') !== 'native')
		return resolveOrdinarySpan(
			runs,
			target.getAttribute('data-harper-gdocs-source'),
			source,
			nativeSnapshot,
			span,
		);
	if (!Array.isArray(runs) || runs.length === 0) return null;
	const masked = nativeSnapshot
		.replace(GOOGLE_DOCS_TABLE_SEPARATORS, '\n')
		.replace(GOOGLE_DOCS_SOFT_BREAKS, ' ');
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
		// This is already a recognized Docs target. Missing or unknown metadata
		// must not turn it into a legacy nearby-text correction target.
		if (!['logical', 'rects', 'native'].includes(sourceMode ?? '')) return [];
		const nativeSnapshot = target.getAttribute('data-harper-gdocs-native-snapshot');
		const mapped =
			['logical', 'rects', 'native'].includes(sourceMode ?? '') ||
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
			(!['native', 'logical', 'rects'].includes(sourceMode ?? '') ||
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
				splitsSurrogate(nativeSnapshot, resolvedSpan.start) ||
				splitsSurrogate(nativeSnapshot, resolvedSpan.end) ||
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

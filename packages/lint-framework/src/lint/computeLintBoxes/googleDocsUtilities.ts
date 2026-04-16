import type { Span } from 'harper.js';
import { leafNodes } from '../domUtils';
import type { UnpackedLint, UnpackedSpan } from '../unpackLint';

type GoogleDocsReplacePayload = {
	start: number;
	end: number;
	replacementText: string;
	expectedText: string;
	beforeContext: string;
	afterContext: string;
};

let googleDocsMeasurementContext: CanvasRenderingContext2D | null | undefined;

function getGoogleDocsMeasurementContext(): CanvasRenderingContext2D | null {
	if (googleDocsMeasurementContext !== undefined) {
		return googleDocsMeasurementContext;
	}

	const canvas = document.createElement('canvas');
	googleDocsMeasurementContext = canvas.getContext('2d');
	return googleDocsMeasurementContext;
}

function getGoogleDocsTextSliceRect(
	host: HTMLElement,
	text: string,
	startOffset: number,
	endOffset: number,
): DOMRect | null {
	const hostRect = host.getBoundingClientRect();
	if (hostRect.width <= 0 || hostRect.height <= 0) {
		return null;
	}

	const safeStart = Math.max(0, Math.min(startOffset, text.length));
	const safeEnd = Math.max(safeStart, Math.min(endOffset, text.length));
	if (safeStart === safeEnd) {
		return null;
	}

	const ctx = getGoogleDocsMeasurementContext();
	if (ctx == null) {
		return hostRect;
	}

	const style = getComputedStyle(host);
	const font =
		style.font && style.font !== ''
			? style.font
			: `${style.fontStyle} ${style.fontVariant} ${style.fontWeight} ${style.fontSize} / ${style.lineHeight} ${style.fontFamily}`;
	ctx.font = font;

	const prefix = text.slice(0, safeStart);
	const slice = text.slice(safeStart, safeEnd);
	const prefixWidth = ctx.measureText(prefix).width;
	const sliceWidth = Math.max(1, ctx.measureText(slice).width);
	const letterSpacing = Number.parseFloat(style.letterSpacing);
	const spacing = Number.isFinite(letterSpacing) ? letterSpacing : 0;
	const spacedPrefixWidth = prefixWidth + Math.max(0, safeStart - 1) * spacing;
	const spacedSliceWidth = sliceWidth + Math.max(0, safeEnd - safeStart - 1) * spacing;
	const x = hostRect.x + Math.min(hostRect.width, Math.max(0, spacedPrefixWidth));
	const width = Math.max(1, Math.min(hostRect.right - x, spacedSliceWidth));

	return new DOMRect(x, hostRect.y, width, hostRect.height);
}

function getCommonPrefixLength(left: string, right: string): number {
	const max = Math.min(left.length, right.length);
	let length = 0;

	while (length < max && left.charCodeAt(length) === right.charCodeAt(length)) {
		length += 1;
	}

	return length;
}

function getCommonSuffixLength(left: string, right: string): number {
	const max = Math.min(left.length, right.length);
	let length = 0;

	while (
		length < max &&
		left.charCodeAt(left.length - 1 - length) === right.charCodeAt(right.length - 1 - length)
	) {
		length += 1;
	}

	return length;
}

function getLongestCommonSubsequenceLength(left: string, right: string): number {
	if (!left || !right) {
		return 0;
	}

	const prev = new Array(right.length + 1).fill(0);
	const next = new Array(right.length + 1).fill(0);

	for (let i = 1; i <= left.length; i += 1) {
		next[0] = 0;

		for (let j = 1; j <= right.length; j += 1) {
			if (left.charCodeAt(i - 1) === right.charCodeAt(j - 1)) {
				next[j] = prev[j - 1] + 1;
			} else {
				next[j] = Math.max(prev[j], next[j - 1]);
			}
		}

		for (let j = 0; j <= right.length; j += 1) {
			prev[j] = next[j];
		}
	}

	return prev[right.length];
}

/**
 * Re-resolves a Google Docs lint span against the bridge's current text.
 *
 * Google Docs text can shift between linting and rendering, so this uses the
 * original span text plus nearby context to find the best current match.
 */
export function resolveGoogleDocsSpan(
	currentSource: string,
	lint: UnpackedLint,
): UnpackedSpan | null {
	const safeStart = Math.max(0, Math.min(lint.span.start, lint.source.length));
	const safeEnd = Math.max(safeStart, Math.min(lint.span.end, lint.source.length));
	const expectedText = lint.source.slice(safeStart, safeEnd);
	if (expectedText.length === 0) {
		return null;
	}

	const directText = currentSource.slice(safeStart, safeEnd);
	if (directText === expectedText) {
		return { start: safeStart, end: safeEnd };
	}

	const spanLength = safeEnd - safeStart;
	const delta = currentSource.length - lint.source.length;
	const expectedStart = Math.max(0, safeStart + delta);

	for (let offset = -12; offset <= 12; offset += 1) {
		const start = expectedStart + offset;
		if (start < 0) {
			continue;
		}

		const end = start + spanLength;
		if (end > currentSource.length) {
			continue;
		}

		if (currentSource.slice(start, end) === expectedText) {
			return { start, end };
		}
	}

	const contextRadius = 64;
	const beforeContext = lint.source.slice(Math.max(0, safeStart - contextRadius), safeStart);
	const afterContext = lint.source.slice(
		safeEnd,
		Math.min(lint.source.length, safeEnd + contextRadius),
	);
	const beforeWindowLength = Math.max(beforeContext.length * 2, beforeContext.length + 64);
	const afterWindowLength = Math.max(afterContext.length * 2, afterContext.length + 64);
	const hits: Array<{ start: number; end: number; score: number }> = [];
	let cursor = 0;

	while (cursor <= currentSource.length) {
		const start = currentSource.indexOf(expectedText, cursor);
		if (start < 0) {
			break;
		}

		const end = start + expectedText.length;
		const candidateBefore = currentSource.slice(Math.max(0, start - beforeWindowLength), start);
		const candidateAfter = currentSource.slice(
			end,
			Math.min(currentSource.length, end + afterWindowLength),
		);
		let score = 0;

		score += getLongestCommonSubsequenceLength(beforeContext, candidateBefore) * 8;
		score += getLongestCommonSubsequenceLength(afterContext, candidateAfter) * 8;
		score += getCommonPrefixLength(beforeContext, candidateBefore) * 4;
		score += getCommonSuffixLength(beforeContext, candidateBefore) * 4;
		score += getCommonPrefixLength(afterContext, candidateAfter) * 4;
		score += getCommonSuffixLength(afterContext, candidateAfter) * 4;
		score -= Math.abs(start - expectedStart) / 1000;
		hits.push({ start, end, score });
		cursor = start + 1;
	}

	if (hits.length === 0) {
		return null;
	}

	hits.sort((left, right) => right.score - left.score);
	return { start: hits[0].start, end: hits[0].end };
}

/**
 * Identifies Harper's hidden Google Docs bridge target element.
 */
export function isGoogleDocsTarget(el: HTMLElement): boolean {
	return el.getAttribute('data-harper-google-docs-target') === 'true';
}

/**
 * Maps a span within the mirrored Google Docs target back to visible client rects.
 *
 * This prefers the positioned bridge spans used for Google Docs text slices and
 * falls back to DOM Range geometry when needed.
 */
export function getGoogleDocsHighlightRects(target: HTMLElement, span: Span): DOMRect[] {
	const children = leafNodes(target);
	const rects: DOMRect[] = [];
	let traversed = 0;

	for (const child of children) {
		const childText = child.textContent ?? '';
		const childLength = childText.length;
		const childStart = traversed;
		const childEnd = childStart + childLength;
		traversed = childEnd;
		const localStart = Math.max(0, span.start - childStart);
		const localEnd = Math.min(childLength, span.end - childStart);

		if (childLength === 0 || childEnd <= span.start || childStart >= span.end) {
			continue;
		}

		const positionedHost = getGoogleDocsPositionedLeafHost(child, target);
		if (positionedHost != null) {
			const rect = getGoogleDocsTextSliceRect(positionedHost, childText, localStart, localEnd);
			if (rect != null && rect.width > 0 && rect.height > 0) {
				rects.push(rect);
				continue;
			}
		}

		const range = document.createRange();
		range.setStart(child, localStart);
		range.setEnd(child, localEnd);
		const rangeRects = Array.from(range.getClientRects()).filter(
			(rect) => rect.width > 0 && rect.height > 0,
		);
		if (rangeRects.length > 0) {
			rects.push(...rangeRects);
			(range as any).detach?.();
			continue;
		}

		const rect = positionedHost?.getBoundingClientRect();
		if (rect != null && rect.width > 0 && rect.height > 0) {
			rects.push(rect);
		}

		(range as any).detach?.();
	}

	return rects;
}

function getGoogleDocsPositionedLeafHost(child: Node, target: HTMLElement): HTMLElement | null {
	let current = child.parentElement;

	while (current != null && current !== target) {
		if (getComputedStyle(current).position === 'absolute') {
			return current;
		}

		current = current.parentElement;
	}

	return null;
}

/**
 * Dispatches a Google Docs replacement through the bridge client exposed on `window`.
 *
 * The payload includes nearby context so the bridge can recover when the original
 * offsets have gone stale by the time the suggestion is applied.
 */
export function replaceGoogleDocsValue(
	span: { start: number; end: number },
	replacementText: string,
	source: string,
) {
	try {
		const safeStart = Math.max(0, Math.min(span.start, source.length));
		const safeEnd = Math.max(safeStart, Math.min(span.end, source.length));
		const expectedText = source.slice(safeStart, safeEnd);
		const contextRadius = 64;
		const beforeContext = source.slice(Math.max(0, safeStart - contextRadius), safeStart);
		const afterContext = source.slice(safeEnd, Math.min(source.length, safeEnd + contextRadius));

		const payload: GoogleDocsReplacePayload = {
			start: span.start,
			end: span.end,
			replacementText,
			expectedText,
			beforeContext,
			afterContext,
		};
		const script = document.createElement('script');
		script.textContent = `(() => {
			const payload = ${JSON.stringify(payload)};
			const normalizeGoogleDocsText = (text) => {
				const raw = String(text ?? '');
				const withoutSentinel = raw.startsWith('\\u0003') ? raw.slice(1) : raw;
				return withoutSentinel.endsWith('\\n') ? withoutSentinel.slice(0, -1) : withoutSentinel;
			};
			const normalizedToRawOffset = (rawText, normalizedOffset) => {
				const raw = String(rawText ?? '');
				const leadingOffset = raw.startsWith('\\u0003') ? 1 : 0;
				const trailingOffset = raw.endsWith('\\n') ? 1 : 0;
				const rawEnd = Math.max(leadingOffset, raw.length - trailingOffset);
				const safeOffset = Math.max(0, Number.isFinite(normalizedOffset) ? normalizedOffset : 0);
				return Math.max(leadingOffset, Math.min(rawEnd, safeOffset + leadingOffset));
			};
			const getCommonPrefixLength = (left, right) => {
				const max = Math.min(left.length, right.length);
				let length = 0;
				while (length < max && left.charCodeAt(length) === right.charCodeAt(length)) {
					length += 1;
				}
				return length;
			};
			const getCommonSuffixLength = (left, right) => {
				const max = Math.min(left.length, right.length);
				let length = 0;
				while (
					length < max &&
					left.charCodeAt(left.length - 1 - length) === right.charCodeAt(right.length - 1 - length)
				) {
					length += 1;
				}
				return length;
			};
			const getLongestCommonSubsequenceLength = (left, right) => {
				if (!left || !right) return 0;
				const previous = new Array(right.length + 1).fill(0);
				const current = new Array(right.length + 1).fill(0);
				for (let i = 1; i <= left.length; i += 1) {
					current[0] = 0;
					for (let j = 1; j <= right.length; j += 1) {
						if (left.charCodeAt(i - 1) === right.charCodeAt(j - 1)) {
							current[j] = previous[j - 1] + 1;
						} else {
							current[j] = Math.max(previous[j], current[j - 1]);
						}
					}
					for (let j = 0; j <= right.length; j += 1) {
						previous[j] = current[j];
					}
				}
				return previous[right.length];
			};
			const resolveReplacementRange = (currentText, start, end, expectedText, beforeContext, afterContext) => {
				const normalizedStart = Math.max(0, Math.min(start, currentText.length));
				const normalizedEnd = Math.max(normalizedStart, Math.min(end, currentText.length));
				const directText = currentText.slice(normalizedStart, normalizedEnd);
				if (!expectedText || directText === expectedText) {
					return { start: normalizedStart, end: normalizedEnd };
				}
				const spanLength = normalizedEnd - normalizedStart;
				for (let delta = -12; delta <= 12; delta += 1) {
					const candidateStart = normalizedStart + delta;
					if (candidateStart < 0) continue;
					const candidateEnd = candidateStart + spanLength;
					if (candidateEnd > currentText.length) continue;
					if (currentText.slice(candidateStart, candidateEnd) === expectedText) {
						return { start: candidateStart, end: candidateEnd };
					}
				}
				const beforeWindowLength = Math.max(beforeContext.length * 2, beforeContext.length + 64);
				const afterWindowLength = Math.max(afterContext.length * 2, afterContext.length + 64);
				const hits = [];
				let cursor = 0;
				while (cursor <= currentText.length) {
					const index = currentText.indexOf(expectedText, cursor);
					if (index < 0) break;
					const indexEnd = index + expectedText.length;
					const candidateBefore = currentText.slice(Math.max(0, index - beforeWindowLength), index);
					const candidateAfter = currentText.slice(indexEnd, Math.min(currentText.length, indexEnd + afterWindowLength));
					let score = 0;
					score += getLongestCommonSubsequenceLength(beforeContext, candidateBefore) * 8;
					score += getLongestCommonSubsequenceLength(afterContext, candidateAfter) * 8;
					score += getCommonPrefixLength(beforeContext, candidateBefore) * 4;
					score += getCommonSuffixLength(beforeContext, candidateBefore) * 4;
					score += getCommonPrefixLength(afterContext, candidateAfter) * 4;
					score += getCommonSuffixLength(afterContext, candidateAfter) * 4;
					score -= Math.abs(index - normalizedStart) / 1000;
					hits.push({ start: index, end: indexEnd, score });
					cursor = index + 1;
				}
				if (hits.length === 0) {
					return { start: normalizedStart, end: normalizedEnd };
				}
				hits.sort((left, right) => right.score - left.score);
				return { start: hits[0].start, end: hits[0].end };
			};
			void window._docs_annotate_getAnnotatedText?.().then((annotated) => {
				if (!annotated || typeof annotated.setSelection !== 'function') {
					return;
				}
				const rawText = String(annotated.getText?.() ?? '');
				const currentText = normalizeGoogleDocsText(rawText);
				const resolvedRange = resolveReplacementRange(
					currentText,
					Number(payload.start),
					Number(payload.end),
					String(payload.expectedText ?? ''),
					String(payload.beforeContext ?? ''),
					String(payload.afterContext ?? ''),
				);
				const rawStart = normalizedToRawOffset(rawText, resolvedRange.start);
				const rawEnd = normalizedToRawOffset(rawText, resolvedRange.end);
				annotated.setSelection(rawStart, rawEnd);
				const iframe = document.querySelector('.docs-texteventtarget-iframe');
				const targetDocument = iframe?.contentDocument;
				if (!targetDocument) {
					return;
				}
				iframe?.focus?.();
				targetDocument.defaultView?.focus?.();
				const target =
					targetDocument.querySelector('[contenteditable="true"]') ??
					targetDocument.activeElement ??
					targetDocument.body ??
					targetDocument.documentElement;
				if (!(target instanceof HTMLElement)) {
					return;
				}
				target.focus?.();
				const dataTransfer = new DataTransfer();
				dataTransfer.items.add(
					payload.replacementText.length === 0 ? '   ' : payload.replacementText,
					'text/plain',
				);
				const pasteEvent = new ClipboardEvent('paste', {
					clipboardData: dataTransfer,
					cancelable: true,
					bubbles: true,
				});
				target.dispatchEvent(pasteEvent);
			}).catch(() => {});
		})();`;
		(document.head || document.documentElement).appendChild(script);
		script.remove();
	} catch {
		// Ignore bridge dispatch failures.
	}
}

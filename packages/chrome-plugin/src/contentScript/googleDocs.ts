import type { LintFramework } from 'lint-framework';
import GoogleDocsBridgeClient from './GoogleDocsBridgeClient';
import {
	createGoogleDocsLineBand,
	extendGoogleDocsLineBand,
	type GoogleDocsLineBand,
	type GoogleDocsRectLayout,
	getGoogleDocsParagraphBreakThreshold,
	rectSharesGoogleDocsLineBand,
	shouldInsertGoogleDocsSpace,
} from './googleDocsLayout';

declare global {
	interface Window {
		__harperGoogleDocsBridgeClient?: GoogleDocsBridgeClient;
	}
}

const GOOGLE_DOCS_TARGET_ID = 'harper-google-docs-target';
const GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID = 'harper-google-docs-main-world-bridge';
const GOOGLE_DOCS_EDITOR_SELECTOR = '.kix-appview-editor';
const GOOGLE_DOCS_RECT_SELECTOR = 'rect[aria-label]';
const GOOGLE_DOCS_SYNCING_ATTR = 'data-harper-gdocs-syncing';
const GOOGLE_DOCS_IGNORED_LAYOUT_REASONS = new Set(['scroll', 'wheel', 'key-scroll']);
const GOOGLE_DOCS_MIN_PARAGRAPH_BREAK_GAP_PX = 6;
const GOOGLE_DOCS_PARAGRAPH_BREAK_RATIO = 0.5;
// biome-ignore lint/suspicious/noControlCharactersInRegex: Observed native Google Docs table separators, masked one code unit at a time.
const GOOGLE_DOCS_TABLE_SEPARATORS = /[\u0010\u0011\u0012\u001c]/gu;
// biome-ignore lint/suspicious/noControlCharactersInRegex: Native control characters cannot own visible source geometry.
const GOOGLE_DOCS_CONTROL_CHARACTERS = /[\u0000-\u001f]/u;

type LayoutRefreshFramework = LintFramework & {
	refreshLayout?: () => void;
};

type GoogleDocsRectSegment = {
	rectNode: SVGRectElement;
	page: Element | null;
	text: string;
	rect: GoogleDocsRectLayout;
};

type GoogleDocsCloneSnapshot = {
	fragment: DocumentFragment;
	text: string;
	source: 'logical' | 'rects' | 'native';
	nativeSnapshot?: string;
	nativeRuns?: Array<[number, number, number]>;
	segmentCount: number;
	signature: string;
};

export function isGoogleDocsPage(): boolean {
	return (
		window.location.hostname === 'docs.google.com' &&
		window.location.pathname.startsWith('/document/')
	);
}

export function createGoogleDocsBridgeSync(fw: LintFramework): () => Promise<void> {
	let bridgeClient: GoogleDocsBridgeClient | null = null;
	let bridgeAttached = false;
	let syncInFlight = false;
	let syncPending = false;
	let lastCloneSignature = '';
	let injectedMainWorldBridge = false;

	function ensureTarget(editor: HTMLElement): HTMLElement {
		let target = document.getElementById(GOOGLE_DOCS_TARGET_ID);

		if (!(target instanceof HTMLElement)) {
			target = document.createElement('div');
			target.id = GOOGLE_DOCS_TARGET_ID;
			target.setAttribute('data-harper-google-docs-target', 'true');
			target.setAttribute('aria-hidden', 'true');
			target.setAttribute('contenteditable', 'false');
			target.setAttribute('data-language', 'plaintext');
			target.style.position = 'absolute';
			target.style.top = '0';
			target.style.left = '0';
			target.style.width = '0';
			target.style.height = '0';
			target.style.overflow = 'visible';
			target.style.pointerEvents = 'none';
			target.style.opacity = '0';
			target.style.zIndex = '-2147483648';
			editor.appendChild(target);
		}

		if (target.parentElement !== editor) {
			editor.appendChild(target);
		}

		return target;
	}

	function ensureMainWorldBridge() {
		if (
			injectedMainWorldBridge ||
			document.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID) != null
		) {
			injectedMainWorldBridge = true;
			return;
		}

		const script = document.createElement('script');
		script.type = 'module';
		script.src = chrome.runtime.getURL('google-docs-bridge.js');
		script.onload = () => script.remove();
		(document.head || document.documentElement).appendChild(script);
		injectedMainWorldBridge = true;
	}

	function bindBridgeClient() {
		if (bridgeClient != null) {
			return;
		}

		bridgeClient = new GoogleDocsBridgeClient(document);
		window.__harperGoogleDocsBridgeClient = bridgeClient;

		bridgeClient.onTextUpdated(() => {
			void syncGoogleDocsBridge();
		});

		bridgeClient.onLayoutChanged((reason) => {
			if (!GOOGLE_DOCS_IGNORED_LAYOUT_REASONS.has(reason)) {
				(fw as LayoutRefreshFramework).refreshLayout?.();
			}
		});
	}

	function disposeBridgeClient() {
		bridgeClient?.dispose();
		bridgeClient = null;
		delete window.__harperGoogleDocsBridgeClient;
	}

	function addSignatureToken(hash: number, token: string): number {
		let nextHash = hash;
		for (let index = 0; index < token.length; index += 1) {
			nextHash = (nextHash * 33 + token.charCodeAt(index)) >>> 0;
		}
		return nextHash;
	}

	function normalizeGoogleDocsRectLabel(label: string): string {
		const parts = label.split(' ');
		let normalized = '';

		for (let index = 0; index < parts.length; index += 1) {
			const part = parts[index];
			if (part === '') {
				normalized += '\u00a0';
				continue;
			}

			normalized += part;
			if (index === parts.length - 1) {
				continue;
			}

			const keepTightRight = /[(["'“‘/_`-]$/u.test(part);
			const keepTightLeft = /^[)\]"'”’/_`—–-]/u.test(parts[index + 1] ?? '');

			if (!keepTightRight && !keepTightLeft) {
				normalized += ' ';
			}
		}

		return normalized;
	}

	function isStandaloneListMarker(text: string): boolean {
		return /^((\d+|[a-zA-Z]|[ivxlcdmIVXLCDM]+)[.)]|[-+*•◦▪‣●☐☑☒])$/u.test(text.trim());
	}

	function appendText(fragment: DocumentFragment, parts: string[], text: string) {
		if (text.length === 0) {
			return;
		}

		parts.push(text);
		fragment.appendChild(document.createTextNode(text));
	}

	function createCloneSpan(segment: GoogleDocsRectSegment, text: string): HTMLSpanElement {
		const span = document.createElement('span');
		span.textContent = text;
		span.style.position = 'absolute';
		span.style.whiteSpace = 'pre';
		span.style.overflow = 'hidden';
		span.style.left = `${segment.rect.left}px`;
		span.style.top = `${segment.rect.top}px`;
		span.style.width = `${Math.max(1, segment.rect.width)}px`;
		span.style.height = `${Math.max(1, segment.rect.height)}px`;
		span.style.lineHeight = `${Math.max(1, segment.rect.height)}px`;

		const fontCss = segment.rectNode.getAttribute('data-font-css');
		if (fontCss) {
			span.style.font = fontCss;
			// Docs zooms the SVG, but data-font-css remains in unscaled SVG units.
			// The mirror uses screen-space geometry, so its font must use the same scale.
			const svgWidth = segment.rectNode.width.baseVal.value;
			const fontSize = Number.parseFloat(span.style.fontSize);
			if (svgWidth > 0 && Number.isFinite(fontSize)) {
				span.style.fontSize = `${fontSize * (segment.rect.width / svgWidth)}px`;
			}
		}

		return span;
	}

	function canIgnoreLogicalGap(text: string): boolean {
		return /^\s*$/u.test(text);
	}

	function segmentsShareVisualLine(
		left: GoogleDocsRectSegment,
		right: GoogleDocsRectSegment,
	): boolean {
		return (
			rectSharesGoogleDocsLineBand(left.rect, createGoogleDocsLineBand(right.rect)) ||
			rectSharesGoogleDocsLineBand(right.rect, createGoogleDocsLineBand(left.rect))
		);
	}

	function compareSegments(left: GoogleDocsRectSegment, right: GoogleDocsRectSegment): number {
		if (segmentsShareVisualLine(left, right)) {
			const horizontalDelta = left.rect.left - right.rect.left;
			if (Math.abs(horizontalDelta) > 1) {
				return horizontalDelta;
			}
		}

		const verticalDelta = left.rect.top - right.rect.top;
		if (Math.abs(verticalDelta) > 1) {
			return verticalDelta;
		}

		const horizontalDelta = left.rect.left - right.rect.left;
		if (Math.abs(horizontalDelta) > 1) {
			return horizontalDelta;
		}

		return 0;
	}

	function collectRectSegments(editor: HTMLElement): GoogleDocsRectSegment[] {
		const editorRect = editor.getBoundingClientRect();
		const segments: GoogleDocsRectSegment[] = [];

		for (const rectNode of Array.from(
			editor.querySelectorAll<SVGRectElement>(GOOGLE_DOCS_RECT_SELECTOR),
		)) {
			const rawLabel = rectNode.getAttribute('aria-label');
			if (!rawLabel) {
				continue;
			}

			const text = normalizeGoogleDocsRectLabel(rawLabel);
			if (text.length === 0) {
				continue;
			}

			const rect = rectNode.getBoundingClientRect();
			if (!Number.isFinite(rect.top) || rect.width <= 0 || rect.height <= 0) {
				continue;
			}

			const top = rect.top - editorRect.top + editor.scrollTop;
			const left = rect.left - editorRect.left + editor.scrollLeft;
			if (!Number.isFinite(top) || !Number.isFinite(left)) {
				continue;
			}

			segments.push({
				rectNode,
				page: rectNode.closest('.kix-page-paginated'),
				text,
				rect: {
					top,
					left,
					width: rect.width,
					height: rect.height,
				},
			});
		}

		segments.sort(compareSegments);
		return segments;
	}

	function shouldRepairCollapsedGap(
		skippedText: string,
		previousText: string,
		nextText: string,
		sharesVisualLine: boolean,
	): boolean {
		return (
			skippedText.length === 0 &&
			!sharesVisualLine &&
			/[\p{L}\p{N}]$/u.test(previousText) &&
			/^[\p{L}\p{N}]/u.test(nextText)
		);
	}

	function normalizeLogicalGap(
		skippedText: string,
		previousText: string,
		nextText: string,
		sharesVisualLine: boolean,
		currentRect: GoogleDocsRectLayout,
		nextRect: GoogleDocsRectLayout | null,
	): string {
		if (shouldRepairCollapsedGap(skippedText, previousText, nextText, sharesVisualLine)) {
			return ' ';
		}

		if (!skippedText.includes('\n')) {
			return skippedText;
		}

		const startsNewTableColumn =
			nextRect != null &&
			rectSharesGoogleDocsLineBand(currentRect, createGoogleDocsLineBand(nextRect)) &&
			nextRect.left - currentRect.left > currentRect.width + 120;

		if (sharesVisualLine || startsNewTableColumn) {
			return skippedText.replace(/\n+/gu, '\n');
		}

		const shouldKeepParagraphBreak =
			!isStandaloneListMarker(previousText) && !isStandaloneListMarker(nextText);

		return skippedText.replace(/\n+/gu, shouldKeepParagraphBreak ? '\n\n' : '\n');
	}

	function shouldInsertSoftWrapSpace(
		previousText: string,
		nextText: string,
		lineGap: number,
		paragraphBreakThreshold: number,
	): boolean {
		return (
			lineGap < paragraphBreakThreshold &&
			/[\p{L}\p{N}]$/u.test(previousText) &&
			/^[\p{Ll}\p{N}]/u.test(nextText)
		);
	}

	function computeSignature(
		source: GoogleDocsCloneSnapshot['source'],
		text: string,
		segments: GoogleDocsRectSegment[],
		editor: HTMLElement,
	): string {
		let hash = 5381;
		hash = addSignatureToken(hash, source);
		hash = addSignatureToken(hash, text);
		hash = addSignatureToken(hash, `${editor.scrollTop}:${editor.scrollLeft}:${segments.length}`);

		for (const segment of segments) {
			hash = addSignatureToken(
				hash,
				`${Math.round(segment.rect.top)}:${Math.round(segment.rect.left)}:${Math.round(segment.rect.width)}:${Math.round(segment.rect.height)}:${segment.text}`,
			);
		}

		return String(hash);
	}

	function buildLogicalSnapshot(
		editor: HTMLElement,
		segments: GoogleDocsRectSegment[],
		logicalText: string,
	): GoogleDocsCloneSnapshot | null {
		if (logicalText.length === 0 || segments.length === 0) {
			return null;
		}

		const fragment = document.createDocumentFragment();
		const parts: string[] = [];
		let cursor = 0;
		let previousText = '';
		let currentLineBand: GoogleDocsLineBand | null = null;

		for (let index = 0; index < segments.length; index += 1) {
			const segment = segments[index];
			const startAtCursor = logicalText.startsWith(segment.text, cursor)
				? cursor
				: logicalText.indexOf(segment.text, cursor);

			if (startAtCursor < 0) {
				return null;
			}

			const skippedText = logicalText.slice(cursor, startAtCursor);
			if (!canIgnoreLogicalGap(skippedText)) {
				return null;
			}

			const sharesVisualLine: boolean =
				currentLineBand != null && rectSharesGoogleDocsLineBand(segment.rect, currentLineBand);
			const normalizedGap = normalizeLogicalGap(
				skippedText,
				previousText,
				segment.text,
				sharesVisualLine,
				segment.rect,
				segments[index + 1]?.rect ?? null,
			);
			appendText(fragment, parts, normalizedGap);

			fragment.appendChild(createCloneSpan(segment, segment.text));
			parts.push(segment.text);
			cursor = startAtCursor + segment.text.length;
			previousText = segment.text;
			currentLineBand =
				currentLineBand == null || !sharesVisualLine
					? createGoogleDocsLineBand(segment.rect)
					: extendGoogleDocsLineBand(currentLineBand, segment.rect);
		}

		const trailingText = logicalText.slice(cursor);
		if (!canIgnoreLogicalGap(trailingText)) {
			return null;
		}

		appendText(fragment, parts, trailingText);
		const text = parts.join('');

		return {
			fragment,
			text,
			source: 'logical',
			segmentCount: segments.length,
			signature: computeSignature('logical', text, segments, editor),
		};
	}

	function getPageBoundarySeparator(
		previous: GoogleDocsRectSegment,
		current: GoogleDocsRectSegment,
		logicalText: string,
	): string {
		// A failed full-document alignment need not discard the logical gap here.
		// Repeated text is safe only when every matching adjacent pair agrees.
		const currentContent = current.text.trimStart();
		const currentPrefix = current.text
			.slice(0, current.text.length - currentContent.length)
			.replace(/\u00a0/gu, ' ');
		let separator: string | null = null;
		let cursor = 0;
		while (cursor < logicalText.length) {
			const start = logicalText.indexOf(previous.text, cursor);
			if (start < 0) break;
			const end = start + previous.text.length;
			const whitespace = /^\s*/u.exec(logicalText.slice(end))?.[0] ?? '';
			if (logicalText.startsWith(currentContent, end + whitespace.length)) {
				// The rectangle already owns its prefix, even if Docs used regular spaces.
				const gap = whitespace.replace(/\u00a0/gu, ' ').endsWith(currentPrefix)
					? whitespace.slice(0, whitespace.length - currentPrefix.length)
					: whitespace;
				const normalized = normalizeLogicalGap(
					gap,
					previous.text,
					current.text,
					false,
					current.rect,
					null,
				);
				if (separator != null && separator !== normalized) {
					separator = null;
					break;
				}
				separator = normalized;
			}
			cursor = start + 1;
		}
		if (separator != null) return separator;

		// Page margins are layout, not paragraph semantics. Without a usable
		// logical gap, join conservatively rather than inventing a paragraph.
		if (/\s$/u.test(previous.text) || /^\s/u.test(current.text)) return '';
		// Straight quotes and apostrophes are ambiguous, not evidence for a tight join.
		if (/[([{“‘/_—–-]$/u.test(previous.text) || /^[,./:;!?%)\]}”’—–-]/u.test(current.text)) {
			return '';
		}
		return ' ';
	}

	function buildRectSnapshot(
		editor: HTMLElement,
		segments: GoogleDocsRectSegment[],
		logicalText: string,
	): GoogleDocsCloneSnapshot {
		const fragment = document.createDocumentFragment();
		const parts: string[] = [];
		let previousText = '';
		let previousRect: GoogleDocsRectLayout | null = null;
		let previousSegment: GoogleDocsRectSegment | null = null;
		let currentLineBand: GoogleDocsLineBand | null = null;

		for (const segment of segments) {
			const sharesVisualLine: boolean =
				currentLineBand != null && rectSharesGoogleDocsLineBand(segment.rect, currentLineBand);
			const startsNewLine: boolean = currentLineBand != null && !sharesVisualLine;

			if (startsNewLine && currentLineBand != null) {
				const lineGap = Math.max(0, segment.rect.top - currentLineBand.bottom);
				const paragraphBreakThreshold = Math.max(
					GOOGLE_DOCS_MIN_PARAGRAPH_BREAK_GAP_PX,
					getGoogleDocsParagraphBreakThreshold(currentLineBand, segment.rect),
					Math.min(currentLineBand.height, segment.rect.height) * GOOGLE_DOCS_PARAGRAPH_BREAK_RATIO,
				);
				const changesPage =
					previousSegment?.page != null &&
					segment.page != null &&
					previousSegment.page !== segment.page;
				const breakText =
					changesPage && previousSegment != null
						? getPageBoundarySeparator(previousSegment, segment, logicalText)
						: shouldInsertSoftWrapSpace(
									previousText,
									segment.text,
									lineGap,
									paragraphBreakThreshold,
								)
							? ' '
							: lineGap >= paragraphBreakThreshold
								? '\n\n'
								: '\n';
				appendText(fragment, parts, breakText);
			} else if (
				previousRect != null &&
				(isStandaloneListMarker(previousText) ||
					shouldInsertGoogleDocsSpace(previousRect, segment.rect, previousText, segment.text))
			) {
				appendText(fragment, parts, ' ');
			}

			fragment.appendChild(createCloneSpan(segment, segment.text));
			parts.push(segment.text);
			previousText = segment.text;
			previousRect = segment.rect;
			previousSegment = segment;
			currentLineBand =
				currentLineBand == null || startsNewLine
					? createGoogleDocsLineBand(segment.rect)
					: extendGoogleDocsLineBand(currentLineBand, segment.rect);
		}

		const text = parts.join('');

		return {
			fragment,
			text,
			source: 'rects',
			segmentCount: segments.length,
			signature: computeSignature('rects', text, segments, editor),
		};
	}

	function buildNativeTableSnapshot(
		editor: HTMLElement,
		segments: GoogleDocsRectSegment[],
		nativeSnapshot: string,
	): GoogleDocsCloneSnapshot {
		// Mask the observed controls without changing source characters. Confirmed
		// paragraph boundaries below can add LF, so lint offsets need explicit runs.
		// This is not a parser for the complete Google Docs control grammar.
		const masked = nativeSnapshot.replace(GOOGLE_DOCS_TABLE_SEPARATORS, '\n');
		const comparable = nativeSnapshot.replace(/\u00a0/gu, ' ');
		const matchesBySegment = new Map<
			GoogleDocsRectSegment,
			{ start: number; end: number; segment: GoogleDocsRectSegment }
		>();
		const labels = new Map(
			segments.map((segment) => [
				segment,
				(segment.rectNode.getAttribute('aria-label') ?? '').replace(/\u00a0/gu, ' '),
			]),
		);
		for (const segment of segments) {
			// Preserve source characters. Only an equal-length space alias is used.
			const label = labels.get(segment)!;
			const start = comparable.indexOf(label);
			if (
				!label.trim() ||
				GOOGLE_DOCS_CONTROL_CHARACTERS.test(label) ||
				start < 0 ||
				comparable.indexOf(label, start + 1) >= 0
			)
				continue;
			matchesBySegment.set(segment, { start, end: start + label.length, segment });
		}
		// A paragraph's contiguous labels can uniquely anchor a repeated short word.
		// Screen order is used only within this group, never between native cells.
		// Multiple full matches are ambiguity, not permission to pick the nearest.
		const groups = new Map<Element, GoogleDocsRectSegment[]>();
		for (const segment of segments) {
			const group = segment.rectNode.closest('g[role="paragraph"]');
			if (group) groups.set(group, [...(groups.get(group) ?? []), segment]);
		}
		for (const groupSegments of groups.values()) {
			const ordered = groupSegments.toSorted(compareSegments);
			if (
				ordered.length < 2 ||
				ordered.some(
					(segment) =>
						!labels.get(segment)!.trim() ||
						GOOGLE_DOCS_CONTROL_CHARACTERS.test(labels.get(segment)!),
				)
			)
				continue;
			const firstLabel = labels.get(ordered[0])!;
			let cursor = 0;
			let unique: Array<{ start: number; end: number; segment: GoogleDocsRectSegment }> | null =
				null;
			while (cursor < comparable.length) {
				const first = comparable.indexOf(firstLabel, cursor);
				if (first < 0) break;
				const sequence = [{ start: first, end: first + firstLabel.length, segment: ordered[0] }];
				for (const segment of ordered.slice(1)) {
					const previousEnd = sequence[sequence.length - 1].end;
					const label = labels.get(segment)!;
					const start = comparable.indexOf(label, previousEnd);
					// biome-ignore lint/suspicious/noControlCharactersInRegex: Only ordinary whitespace may separate labels inside one native region.
					if (start < 0 || !/^[	\r\n ]*$/u.test(comparable.slice(previousEnd, start))) break;
					sequence.push({ start, end: start + label.length, segment });
				}
				if (sequence.length === ordered.length) {
					if (unique != null) {
						unique = null;
						break;
					}
					unique = sequence;
				}
				cursor = first + 1;
			}
			for (const match of unique ?? []) matchesBySegment.set(match.segment, match);
		}
		const matches = Array.from(matchesBySegment.values());
		// Contradictory geometry is local ambiguity, never a reason to lint visual
		// table prose. Both overlapping assignments remain non-geometric gaps.
		const resolved = matches.filter(
			(match) =>
				!matches.some(
					(other) => other !== match && other.start < match.end && other.end > match.start,
				),
		);
		resolved.sort((left, right) => left.start - right.start);
		const paragraphBreaks = new Set<number>();
		for (let index = 1; index < resolved.length; index += 1) {
			const previous = resolved[index - 1];
			const current = resolved[index];
			const previousGroup = previous.segment.rectNode.closest('g[role="paragraph"]');
			const currentGroup = current.segment.rectNode.closest('g[role="paragraph"]');
			const gap = nativeSnapshot.slice(previous.end, current.start);
			// A native LF between distinct, anchored paragraph groups is semantic.
			// A wrapped line in the same group is not. Table-control gaps already
			// contain their separators and must not be globally newline-normalized.
			if (
				previousGroup != null &&
				currentGroup != null &&
				previousGroup !== currentGroup &&
				// biome-ignore lint/suspicious/noControlCharactersInRegex: Only native paragraph whitespace may separate anchored groups.
				/^[	\r\n ]*$/u.test(gap) &&
				gap.includes('\n') &&
				!gap.includes('\n\n')
			) {
				paragraphBreaks.add(previous.end + gap.indexOf('\n') + 1);
			}
		}
		// Native table markers also separate regions. Existing consecutive masked
		// LF already form a paragraph break and stay unchanged.
		// biome-ignore lint/suspicious/noControlCharactersInRegex: Only observed native table controls and adjacent whitespace define these boundaries.
		for (const gap of nativeSnapshot.matchAll(/[	\r\n \u0010\u0011\u0012\u001c]+/gu)) {
			const end = gap.index + gap[0].length;
			if (
				gap.index > 0 &&
				end < nativeSnapshot.length &&
				gap[0].search(GOOGLE_DOCS_TABLE_SEPARATORS) >= 0 &&
				!gap[0].replace(GOOGLE_DOCS_TABLE_SEPARATORS, '\n').includes('\n\n')
			)
				paragraphBreaks.add(gap.index + gap[0].search(GOOGLE_DOCS_TABLE_SEPARATORS) + 1);
		}
		const breaks = Array.from(paragraphBreaks).sort((left, right) => left - right);
		// Each run is [projection start, native start, UTF-16 length]. Inserted
		// separators are deliberately outside runs and cannot own an edit range.
		const nativeRuns: Array<[number, number, number]> = [];
		const parts: string[] = [];
		let nativeCursor = 0;
		let projectionCursor = 0;
		for (const end of [...breaks, nativeSnapshot.length]) {
			const slice = masked.slice(nativeCursor, end);
			nativeRuns.push([projectionCursor, nativeCursor, slice.length]);
			parts.push(slice);
			projectionCursor += slice.length;
			if (end < nativeSnapshot.length) {
				parts.push('\n');
				projectionCursor += 1;
			}
			nativeCursor = end;
		}
		const text = parts.join('');
		const projectOffset = (offset: number) => offset + breaks.filter((end) => end <= offset).length;
		const fragment = document.createDocumentFragment();
		let cursor = 0;
		for (const { start, end, segment } of resolved) {
			const projectedStart = projectOffset(start);
			const projectedEnd = projectOffset(end);
			fragment.appendChild(document.createTextNode(text.slice(cursor, projectedStart)));
			const span = createCloneSpan(segment, text.slice(projectedStart, projectedEnd));
			span.setAttribute('data-harper-gdocs-native-start', String(start));
			fragment.appendChild(span);
			cursor = projectedEnd;
		}
		fragment.appendChild(document.createTextNode(text.slice(cursor)));
		return {
			fragment,
			text,
			source: 'native',
			nativeSnapshot,
			nativeRuns,
			segmentCount: resolved.length,
			// Mapping identity includes exact ranges, geometry, and font. Legacy
			// normalized labels can stay equal when native matching changes.
			signature: JSON.stringify([
				editor.scrollTop,
				editor.scrollLeft,
				nativeRuns,
				resolved.map(({ start, end, segment }) => [
					start,
					end,
					segment.rect,
					segment.rectNode.getAttribute('data-font-css'),
				]),
			]),
		};
	}

	function buildSnapshot(editor: HTMLElement): GoogleDocsCloneSnapshot {
		const segments = collectRectSegments(editor);
		// The bridge removes only its leading sentinel and final newline. Keep the
		// exact remaining code units for mapped requests; CRLF conversion shifts them.
		const nativeText = document.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID)?.textContent ?? '';
		if (nativeText.search(GOOGLE_DOCS_TABLE_SEPARATORS) >= 0) {
			return buildNativeTableSnapshot(editor, segments, nativeText);
		}
		const logicalText = nativeText.replace(/\r\n?/gu, '\n');
		return (
			buildLogicalSnapshot(editor, segments, logicalText) ??
			buildRectSnapshot(editor, segments, logicalText)
		);
	}

	function applySnapshot(target: HTMLElement, snapshot: GoogleDocsCloneSnapshot): boolean {
		const previousNative = target.getAttribute('data-harper-gdocs-native-snapshot');
		if (
			snapshot.signature === lastCloneSignature &&
			target.textContent === snapshot.text &&
			previousNative === (snapshot.nativeSnapshot ?? null)
		) {
			return false;
		}

		target.replaceChildren(snapshot.fragment);
		if (snapshot.nativeSnapshot != null) {
			target.setAttribute('data-harper-gdocs-native-snapshot', snapshot.nativeSnapshot);
			target.setAttribute('data-harper-gdocs-native-runs', JSON.stringify(snapshot.nativeRuns));
		} else {
			target.removeAttribute('data-harper-gdocs-native-snapshot');
			target.removeAttribute('data-harper-gdocs-native-runs');
		}
		target.setAttribute('data-harper-gdocs-source', snapshot.source);
		target.setAttribute('data-harper-gdocs-segments', String(snapshot.segmentCount));
		lastCloneSignature = snapshot.signature;
		return true;
	}

	async function syncGoogleDocsBridge() {
		if (!isGoogleDocsPage()) {
			disposeBridgeClient();
			bridgeAttached = false;
			return;
		}

		if (syncInFlight) {
			syncPending = true;
			return;
		}

		syncInFlight = true;

		try {
			ensureMainWorldBridge();
			bindBridgeClient();

			const editor = document.querySelector(GOOGLE_DOCS_EDITOR_SELECTOR);
			if (!(editor instanceof HTMLElement)) {
				return;
			}

			const target = ensureTarget(editor);
			editor.setAttribute(GOOGLE_DOCS_SYNCING_ATTR, 'true');

			const changed = applySnapshot(target, buildSnapshot(editor));
			if (!bridgeAttached) {
				await fw.addTarget(target);
				bridgeAttached = true;
			}

			if (changed) {
				await fw.update();
			}
		} catch (error) {
			console.error('Failed to sync Google Docs bridge text', error);
		} finally {
			const editor = document.querySelector(GOOGLE_DOCS_EDITOR_SELECTOR);
			if (editor instanceof HTMLElement) {
				// Snapshot replacement is complete before the queued animation-frame render.
				// Keeping this flag set delays that render until an unrelated later update.
				editor.removeAttribute(GOOGLE_DOCS_SYNCING_ATTR);
			}

			syncInFlight = false;
			if (syncPending) {
				syncPending = false;
				void syncGoogleDocsBridge();
			}
		}
	}

	return syncGoogleDocsBridge;
}

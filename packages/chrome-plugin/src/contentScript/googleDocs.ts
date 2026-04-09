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

type LayoutRefreshFramework = LintFramework & {
	refreshLayout?: () => void;
};

type GoogleDocsRectSegment = {
	rectNode: SVGRectElement;
	text: string;
	rect: GoogleDocsRectLayout;
};

type GoogleDocsCloneSnapshot = {
	fragment: DocumentFragment;
	text: string;
	source: 'logical' | 'rects';
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
	let syncingClearTimer: number | null = null;
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

	function buildRectSnapshot(
		editor: HTMLElement,
		segments: GoogleDocsRectSegment[],
	): GoogleDocsCloneSnapshot {
		const fragment = document.createDocumentFragment();
		const parts: string[] = [];
		let previousText = '';
		let previousRect: GoogleDocsRectLayout | null = null;
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
				const breakText = shouldInsertSoftWrapSpace(
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

	function buildSnapshot(editor: HTMLElement): GoogleDocsCloneSnapshot {
		const segments = collectRectSegments(editor);
		const logicalText =
			document
				.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID)
				?.textContent?.replace(/\r\n?/gu, '\n') ?? '';

		return (
			buildLogicalSnapshot(editor, segments, logicalText) ?? buildRectSnapshot(editor, segments)
		);
	}

	function applySnapshot(target: HTMLElement, snapshot: GoogleDocsCloneSnapshot): boolean {
		if (snapshot.signature === lastCloneSignature && target.textContent === snapshot.text) {
			return false;
		}

		target.replaceChildren(snapshot.fragment);
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
			if (syncingClearTimer != null) {
				window.clearTimeout(syncingClearTimer);
				syncingClearTimer = null;
			}
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
				syncingClearTimer = window.setTimeout(() => {
					editor.removeAttribute(GOOGLE_DOCS_SYNCING_ATTR);
					syncingClearTimer = null;
				}, 150);
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

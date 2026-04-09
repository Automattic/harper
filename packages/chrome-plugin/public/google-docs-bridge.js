import { GoogleDocsBridgeRequestHandler } from './google-docs-bridge-request-handler.js';

(() => {
	/**
	 * @typedef {{ x: number, y: number, width: number, height: number }} Rect
	 * @typedef {{ start: number, end: number }} SelectionEndpoints
	 * @typedef {{
	 *   getText: () => string,
	 *   setSelection: (start: number, end: number) => void,
	 *   getSelection?: () => Array<Record<string, unknown>>
	 * }} AnnotatedText
	 * @typedef {{ node: Window | Element, left: number, top: number }} ScrollSnapshot
	 */

	const PROTOCOL_VERSION = 'harper-gdocs-bridge/v1';
	const BRIDGE_ID = 'harper-google-docs-main-world-bridge';
	const POLL_INTERVAL_MS = 125;
	const EDITOR_SELECTOR = '.kix-appview-editor';
	const EDITOR_CONTAINER_SELECTOR = '.kix-appview-editor-container';
	const DOCS_EDITOR_SELECTOR = '#docs-editor';
	const CARET_SELECTOR = '.kix-cursor-caret';
	const TEXT_EVENT_IFRAME_SELECTOR = '.docs-texteventtarget-iframe';
	const LAYOUT_EPOCH_ATTR = 'data-harper-layout-epoch';
	const LAYOUT_REASON_ATTR = 'data-harper-layout-reason';
	const SELECTION_START_ATTR = 'data-harper-selection-start';
	const SELECTION_END_ATTR = 'data-harper-selection-end';
	const EVENT_NOTIFICATION = 'harper:gdocs:notification';
	const EVENT_TEXT_UPDATED = 'harper:gdocs:text-updated';
	const EVENT_LAYOUT_CHANGED = 'harper:gdocs:layout-changed';
	const EVENT_GET_RECTS = 'harper:gdocs:get-rects';
	const EVENT_REPLACE = 'harper:gdocs:replace';
	const CARET_DIRECTION_THRESHOLD = 20;
	const CARET_CHOICE_STALE_MS = 1500;

	let bridgeNode = /** @type {HTMLElement | null} */ (document.getElementById(BRIDGE_ID));
	let currentAnnotated = /** @type {AnnotatedText | null} */ (null);
	let observedEditor = /** @type {HTMLElement | null} */ (null);
	let layoutObserver = /** @type {MutationObserver | null} */ (null);
	let resizeObserver = /** @type {ResizeObserver | null} */ (null);
	let layoutEpoch = 0;
	let layoutScheduled = false;
	let lastCaretChoice = /** @type {{ rect: Rect, position: number, at: number } | null} */ (null);
	let textSyncInFlight = false;

	function ensureBridgeNode() {
		if (bridgeNode instanceof HTMLElement) {
			return bridgeNode;
		}

		const nextBridge = document.createElement('div');
		nextBridge.id = BRIDGE_ID;
		nextBridge.setAttribute('aria-hidden', 'true');
		nextBridge.style.display = 'none';
		document.documentElement.appendChild(nextBridge);
		bridgeNode = nextBridge;
		return nextBridge;
	}

	function dispatchEvent(name, detail) {
		try {
			document.dispatchEvent(new CustomEvent(name, { detail }));
		} catch {}
	}

	function dispatchNotification(kind, detail) {
		dispatchEvent(EVENT_NOTIFICATION, {
			protocol: PROTOCOL_VERSION,
			notification: {
				kind,
				...detail,
			},
		});
	}

	function scheduleLayoutUpdate(reason) {
		if (layoutScheduled) {
			return;
		}

		layoutScheduled = true;
		queueMicrotask(() => {
			layoutScheduled = false;
			layoutEpoch += 1;
			const bridge = ensureBridgeNode();
			bridge.setAttribute(LAYOUT_EPOCH_ATTR, String(layoutEpoch));
			bridge.setAttribute(LAYOUT_REASON_ATTR, String(reason));
			dispatchEvent(EVENT_LAYOUT_CHANGED, { layoutEpoch, reason });
			dispatchNotification('layoutChanged', { layoutEpoch, reason });
		});
	}

	function normalizeGoogleDocsText(text) {
		const raw = String(text ?? '');
		const withoutSentinel = raw.startsWith('\u0003') ? raw.slice(1) : raw;
		return withoutSentinel.endsWith('\n') ? withoutSentinel.slice(0, -1) : withoutSentinel;
	}

	function normalizedToRawOffset(rawText, normalizedOffset) {
		const raw = String(rawText ?? '');
		const leadingOffset = raw.startsWith('\u0003') ? 1 : 0;
		const trailingOffset = raw.endsWith('\n') ? 1 : 0;
		const rawEnd = Math.max(leadingOffset, raw.length - trailingOffset);
		const safeOffset = Math.max(0, Number.isFinite(normalizedOffset) ? normalizedOffset : 0);

		return Math.max(leadingOffset, Math.min(rawEnd, safeOffset + leadingOffset));
	}

	function rawToNormalizedOffset(rawText, rawOffset) {
		const raw = String(rawText ?? '');
		const leadingOffset = raw.startsWith('\u0003') ? 1 : 0;
		const trailingOffset = raw.endsWith('\n') ? 1 : 0;
		const rawEnd = Math.max(leadingOffset, raw.length - trailingOffset);
		const safeOffset = Math.max(
			leadingOffset,
			Math.min(rawEnd, Number.isFinite(rawOffset) ? rawOffset : leadingOffset),
		);

		return Math.max(0, safeOffset - leadingOffset);
	}

	function asFiniteNumber(value) {
		const number = Number(value);
		return Number.isFinite(number) ? number : null;
	}

	function getSelectionEndpoints(selection) {
		if (!selection || typeof selection !== 'object') {
			return null;
		}

		for (const [startKey, endKey] of [
			['anchor', 'focus'],
			['base', 'extent'],
			['start', 'end'],
		]) {
			const start = asFiniteNumber(selection[startKey]);
			const end = asFiniteNumber(selection[endKey]);
			if (start != null && end != null) {
				return { start, end };
			}
		}

		return null;
	}

	function getAnnotatedTextApi() {
		return typeof window._docs_annotate_getAnnotatedText === 'function'
			? window._docs_annotate_getAnnotatedText
			: null;
	}

	async function syncText() {
		if (textSyncInFlight) {
			return;
		}

		textSyncInFlight = true;

		try {
			const getAnnotatedText = getAnnotatedTextApi();
			if (!getAnnotatedText) {
				return;
			}

			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.getText !== 'function') {
				return;
			}

			currentAnnotated = annotated;
			window.__harperGoogleDocsAnnotatedText = annotated;

			const rawText = annotated.getText();
			const normalizedText = normalizeGoogleDocsText(rawText);
			const bridge = ensureBridgeNode();
			const selection = getSelectionEndpoints(annotated.getSelection?.()?.[0]);

			if (selection) {
				bridge.setAttribute(
					SELECTION_START_ATTR,
					String(rawToNormalizedOffset(rawText, selection.start)),
				);
				bridge.setAttribute(
					SELECTION_END_ATTR,
					String(rawToNormalizedOffset(rawText, selection.end)),
				);
			} else {
				bridge.removeAttribute(SELECTION_START_ATTR);
				bridge.removeAttribute(SELECTION_END_ATTR);
			}

			if (bridge.textContent !== normalizedText) {
				bridge.textContent = normalizedText;
				dispatchEvent(EVENT_TEXT_UPDATED, { length: normalizedText.length });
				dispatchNotification('textUpdated', { length: normalizedText.length });
			}
		} catch {
			// Ignore bridge sync failures. The next poll or mutation will retry.
		} finally {
			textSyncInFlight = false;
		}
	}

	function disconnectLayoutObservers() {
		layoutObserver?.disconnect();
		layoutObserver = null;
		resizeObserver?.disconnect();
		resizeObserver = null;
		observedEditor = null;
	}

	function bindLayoutObservers() {
		const editor = document.querySelector(EDITOR_SELECTOR);
		if (!(editor instanceof HTMLElement)) {
			disconnectLayoutObservers();
			return;
		}

		if (editor === observedEditor) {
			return;
		}

		disconnectLayoutObservers();
		observedEditor = editor;
		scheduleLayoutUpdate('init');

		layoutObserver = new MutationObserver((mutations) => {
			for (const mutation of mutations) {
				if (mutation.type === 'childList' || mutation.type === 'attributes') {
					scheduleLayoutUpdate('mutation');
					break;
				}
			}
		});
		layoutObserver.observe(editor, {
			subtree: true,
			childList: true,
			attributes: true,
			attributeFilter: ['style', 'class'],
		});

		if (typeof ResizeObserver !== 'undefined') {
			resizeObserver = new ResizeObserver(() => {
				scheduleLayoutUpdate('resize');
			});
			resizeObserver.observe(editor);
		}
	}

	function snapshotScroll() {
		/** @type {ScrollSnapshot[]} */
		const snapshots = [{ node: window, left: window.scrollX, top: window.scrollY }];
		const selectors = [EDITOR_SELECTOR, EDITOR_CONTAINER_SELECTOR, DOCS_EDITOR_SELECTOR];

		for (const selector of selectors) {
			const node = document.querySelector(selector);
			if (!(node instanceof Element) || snapshots.some((entry) => entry.node === node)) {
				continue;
			}

			snapshots.push({
				node,
				left: node.scrollLeft,
				top: node.scrollTop,
			});
		}

		return snapshots;
	}

	function restoreScroll(snapshots) {
		for (const snapshot of snapshots) {
			if (snapshot.node === window) {
				if (window.scrollX !== snapshot.left || window.scrollY !== snapshot.top) {
					window.scrollTo(snapshot.left, snapshot.top);
				}
				continue;
			}

			if (!(snapshot.node instanceof Element)) {
				continue;
			}

			if (snapshot.node.scrollLeft !== snapshot.left) {
				snapshot.node.scrollLeft = snapshot.left;
			}

			if (snapshot.node.scrollTop !== snapshot.top) {
				snapshot.node.scrollTop = snapshot.top;
			}
		}
	}

	function withSuppressedScrolling(fn) {
		/** @type {Array<() => void>} */
		const restorers = [];
		const noop = () => {};

		const tryOverride = (target, key) => {
			if (!target || typeof target[key] !== 'function') {
				return;
			}

			const original = target[key];
			try {
				target[key] = noop;
				restorers.push(() => {
					try {
						target[key] = original;
					} catch {}
				});
			} catch {}
		};

		tryOverride(window, 'scrollTo');
		tryOverride(window, 'scrollBy');
		tryOverride(Element.prototype, 'scrollIntoView');
		tryOverride(Element.prototype, 'scrollTo');
		tryOverride(Element.prototype, 'scrollBy');

		try {
			return fn();
		} finally {
			for (let index = restorers.length - 1; index >= 0; index -= 1) {
				restorers[index]();
			}
		}
	}

	function getCaretRect(annotated, position) {
		annotated.setSelection(position, position);
		const epsilon = 0.5;
		const viewportWidth = window.innerWidth;
		const viewportHeight = window.innerHeight;
		/** @type {Rect[]} */
		const carets = Array.from(document.querySelectorAll(CARET_SELECTOR))
			.map((caret) => {
				const rect = caret.getBoundingClientRect();
				const style = window.getComputedStyle(caret);
				const onScreen =
					rect.left >= -epsilon &&
					rect.top >= -epsilon &&
					rect.right <= viewportWidth + epsilon &&
					rect.bottom <= viewportHeight + epsilon;

				if (
					rect.width <= 0 ||
					rect.height <= 0 ||
					style.display === 'none' ||
					style.visibility === 'hidden' ||
					style.opacity === '0' ||
					!onScreen
				) {
					return null;
				}

				return {
					x: rect.x,
					y: rect.y,
					width: rect.width,
					height: rect.height,
				};
			})
			.filter((rect) => rect != null);

		if (carets.length === 0) {
			return null;
		}

		const visiblePageCarets = carets.filter((rect) => rect.x > 100);
		const pool = visiblePageCarets.length > 0 ? visiblePageCarets : carets;
		const now = Date.now();

		if (!lastCaretChoice || now - lastCaretChoice.at > CARET_CHOICE_STALE_MS) {
			const choice = pool.reduce((best, rect) => (rect.x < best.x ? rect : best), pool[0]);
			lastCaretChoice = { rect: choice, position, at: now };
			return choice;
		}

		const positionDelta = position - lastCaretChoice.position;
		let choice = null;

		if (positionDelta > CARET_DIRECTION_THRESHOLD) {
			const lowerRects = pool.filter((rect) => rect.y >= lastCaretChoice.rect.y - 2);
			if (lowerRects.length > 0) {
				choice = lowerRects.reduce((best, rect) => {
					if (rect.y > best.y + 1) return rect;
					if (Math.abs(rect.y - best.y) <= 1 && rect.x < best.x) return rect;
					return best;
				}, lowerRects[0]);
			}
		} else if (positionDelta < -CARET_DIRECTION_THRESHOLD) {
			const upperRects = pool.filter((rect) => rect.y <= lastCaretChoice.rect.y + 2);
			if (upperRects.length > 0) {
				choice = upperRects.reduce((best, rect) => {
					if (rect.y < best.y - 1) return rect;
					if (Math.abs(rect.y - best.y) <= 1 && rect.x < best.x) return rect;
					return best;
				}, upperRects[0]);
			}
		}

		if (!choice) {
			choice = pool.reduce((best, rect) => {
				const bestScore =
					Math.abs(best.y - lastCaretChoice.rect.y) * 4 + Math.abs(best.x - lastCaretChoice.rect.x);
				const rectScore =
					Math.abs(rect.y - lastCaretChoice.rect.y) * 4 + Math.abs(rect.x - lastCaretChoice.rect.x);
				return rectScore < bestScore ? rect : best;
			}, pool[0]);
		}

		lastCaretChoice = { rect: choice, position, at: now };
		return choice;
	}

	function restoreSelection(annotated, selection) {
		if (!selection) {
			return;
		}

		try {
			annotated.setSelection(selection.start, selection.end);
		} catch {}
	}

	function isSpanNearSelection(start, end, selection) {
		if (!selection) {
			return false;
		}

		const spanStart = Math.max(0, Math.min(start, end));
		const spanEnd = Math.max(spanStart, Math.max(start, end));
		const selectionStart = Math.min(selection.start, selection.end);
		const selectionEnd = Math.max(selection.start, selection.end);
		const maxDistance = 2000;

		if (spanStart <= selectionEnd && spanEnd >= selectionStart) {
			return true;
		}

		if (spanEnd < selectionStart) {
			return selectionStart - spanEnd <= maxDistance;
		}

		return spanStart - selectionEnd <= maxDistance;
	}

	function getCommonPrefixLength(left, right) {
		const maxLength = Math.min(left.length, right.length);
		let index = 0;

		while (index < maxLength && left.charCodeAt(index) === right.charCodeAt(index)) {
			index += 1;
		}

		return index;
	}

	function getCommonSuffixLength(left, right) {
		const maxLength = Math.min(left.length, right.length);
		let index = 0;

		while (
			index < maxLength &&
			left.charCodeAt(left.length - 1 - index) === right.charCodeAt(right.length - 1 - index)
		) {
			index += 1;
		}

		return index;
	}

	function getLongestCommonSubsequenceLength(left, right) {
		if (!left || !right) {
			return 0;
		}

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
	}

	function resolveReplacementRange(
		currentText,
		start,
		end,
		expectedText,
		beforeContext,
		afterContext,
	) {
		const normalizedStart = Math.max(0, Math.min(start, currentText.length));
		const normalizedEnd = Math.max(normalizedStart, Math.min(end, currentText.length));
		const directText = currentText.slice(normalizedStart, normalizedEnd);

		if (!expectedText || directText === expectedText) {
			return {
				start: normalizedStart,
				end: normalizedEnd,
			};
		}

		const spanLength = normalizedEnd - normalizedStart;

		for (let delta = -12; delta <= 12; delta += 1) {
			const candidateStart = normalizedStart + delta;
			if (candidateStart < 0) {
				continue;
			}

			const candidateEnd = candidateStart + spanLength;
			if (candidateEnd > currentText.length) {
				continue;
			}

			if (currentText.slice(candidateStart, candidateEnd) === expectedText) {
				return {
					start: candidateStart,
					end: candidateEnd,
				};
			}
		}

		const beforeWindowLength = Math.max(beforeContext.length * 2, beforeContext.length + 64);
		const afterWindowLength = Math.max(afterContext.length * 2, afterContext.length + 64);
		const hits = [];
		let cursor = 0;

		while (cursor <= currentText.length) {
			const index = currentText.indexOf(expectedText, cursor);
			if (index < 0) {
				break;
			}

			const indexEnd = index + expectedText.length;
			const candidateBefore = currentText.slice(Math.max(0, index - beforeWindowLength), index);
			const candidateAfter = currentText.slice(
				indexEnd,
				Math.min(currentText.length, indexEnd + afterWindowLength),
			);
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
			return {
				start: normalizedStart,
				end: normalizedEnd,
			};
		}

		hits.sort((left, right) => right.score - left.score);
		return {
			start: hits[0].start,
			end: hits[0].end,
		};
	}

	async function handleGetRectsRequest(request) {
		const annotated = currentAnnotated;
		if (!annotated || typeof annotated.setSelection !== 'function') {
			return { kind: 'getRects', rects: [] };
		}

		const rawText = annotated.getText?.() ?? '';
		const normalizedStart = Number(request.start);
		const normalizedEnd = Number(request.end);
		const rawStart = normalizedToRawOffset(rawText, normalizedStart);
		const rawEnd = normalizedToRawOffset(rawText, normalizedEnd);
		const currentSelection = getSelectionEndpoints(annotated.getSelection?.()?.[0]);
		const previousSelection = currentSelection
			? {
					start: rawToNormalizedOffset(rawText, currentSelection.start),
					end: rawToNormalizedOffset(rawText, currentSelection.end),
				}
			: null;

		if (!isSpanNearSelection(normalizedStart, normalizedEnd, previousSelection)) {
			return { kind: 'getRects', rects: [] };
		}

		const scrollSnapshots = snapshotScroll();
		try {
			const spanStart = Math.max(0, Math.min(rawStart, rawEnd));
			const spanEnd = Math.max(spanStart, Math.max(rawStart, rawEnd));
			const { startRect, endRect } = withSuppressedScrolling(() => ({
				startRect: getCaretRect(annotated, spanStart),
				endRect: getCaretRect(annotated, spanEnd),
			}));

			/** @type {Rect[]} */
			const rects = [];

			if (startRect && endRect && Math.abs(startRect.y - endRect.y) < 6) {
				rects.push({
					x: Math.min(startRect.x, endRect.x),
					y: startRect.y,
					width: Math.max(4, Math.abs(endRect.x - startRect.x)),
					height: startRect.height,
				});
			} else if (startRect) {
				rects.push({
					x: startRect.x,
					y: startRect.y,
					width: 8,
					height: startRect.height,
				});
			}

			return {
				kind: 'getRects',
				rects,
			};
		} finally {
			restoreSelection(annotated, currentSelection);
			restoreScroll(scrollSnapshots);
			requestAnimationFrame(() => restoreScroll(scrollSnapshots));
		}
	}

	async function handleReplaceTextRequest(request) {
		const getAnnotatedText = getAnnotatedTextApi();
		if (!getAnnotatedText) {
			return { kind: 'replaceText', applied: false };
		}

		const annotated = await getAnnotatedText();
		if (!annotated || typeof annotated.setSelection !== 'function') {
			return { kind: 'replaceText', applied: false };
		}

		currentAnnotated = annotated;
		window.__harperGoogleDocsAnnotatedText = annotated;

		const replacementText = String(request.replacementText ?? '');
		const rawText = annotated.getText?.() ?? '';
		const currentText = normalizeGoogleDocsText(rawText);
		const resolvedRange = resolveReplacementRange(
			currentText,
			Number(request.start),
			Number(request.end),
			String(request.expectedText ?? ''),
			String(request.beforeContext ?? ''),
			String(request.afterContext ?? ''),
		);
		const rawStart = normalizedToRawOffset(rawText, resolvedRange.start);
		const rawEnd = normalizedToRawOffset(rawText, resolvedRange.end);

		annotated.setSelection(rawStart, rawEnd);

		const iframe = document.querySelector(TEXT_EVENT_IFRAME_SELECTOR);
		const targetDocument = iframe?.contentDocument;
		const target =
			targetDocument?.activeElement ?? targetDocument?.body ?? targetDocument?.documentElement;
		if (!target) {
			return { kind: 'replaceText', applied: false };
		}

		iframe?.focus?.();
		targetDocument?.defaultView?.focus?.();
		target.focus?.();

		const expectedNextText =
			currentText.slice(0, resolvedRange.start) +
			replacementText +
			currentText.slice(resolvedRange.end);

		const didApplyReplacement = async () => {
			const nextAnnotated = await getAnnotatedText();
			return normalizeGoogleDocsText(nextAnnotated?.getText?.()) === expectedNextText;
		};

		if (targetDocument?.execCommand?.('insertText', false, replacementText)) {
			await new Promise((resolve) => setTimeout(resolve, 0));
			if (await didApplyReplacement()) {
				queueMicrotask(() => {
					void syncText();
				});
				return { kind: 'replaceText', applied: true };
			}
		}

		const dataTransfer = new DataTransfer();
		dataTransfer.setData('text/plain', replacementText);
		target.dispatchEvent(
			new ClipboardEvent('paste', {
				clipboardData: dataTransfer,
				cancelable: true,
				bubbles: true,
			}),
		);

		await new Promise((resolve) => setTimeout(resolve, 0));
		const applied = await didApplyReplacement();
		if (applied) {
			queueMicrotask(() => {
				void syncText();
			});
		}

		return {
			kind: 'replaceText',
			applied,
		};
	}

	ensureBridgeNode();

	const requestHandler = new GoogleDocsBridgeRequestHandler({
		onGetRectsRequest: handleGetRectsRequest,
		onReplaceTextRequest: handleReplaceTextRequest,
	});
	requestHandler.start();

	document.addEventListener(EVENT_GET_RECTS, (event) => {
		const detail = /** @type {CustomEvent} */ (event).detail ?? {};
		const requestId = String(detail.requestId ?? '');
		if (!requestId) {
			return;
		}

		void handleGetRectsRequest(detail).then((response) => {
			ensureBridgeNode().setAttribute(
				`data-harper-rects-${requestId}`,
				JSON.stringify(response.rects),
			);
		});
	});

	document.addEventListener(EVENT_REPLACE, (event) => {
		const detail = /** @type {CustomEvent} */ (event).detail ?? {};
		void handleReplaceTextRequest(detail);
	});

	window.addEventListener(
		'resize',
		() => {
			scheduleLayoutUpdate('resize');
		},
		{ passive: true },
	);
	window.addEventListener(
		'scroll',
		() => {
			scheduleLayoutUpdate('scroll');
		},
		{ passive: true, capture: true },
	);
	document.addEventListener(
		'wheel',
		() => {
			scheduleLayoutUpdate('wheel');
		},
		{ passive: true, capture: true },
	);

	void syncText();
	bindLayoutObservers();
	window.setInterval(() => {
		void syncText();
		bindLayoutObservers();
	}, POLL_INTERVAL_MS);
})();

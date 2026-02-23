(() => {
	/**
	 * @typedef {{ x: number, y: number, width: number, height: number }} Rect
	 */

	/**
	 * @typedef {{ start: number, end: number }} SelectionEndpoints
	 */

	/**
	 * @typedef {{
	 *   getText: () => string,
	 *   setSelection: (start: number, end: number) => void,
	 *   getSelection?: () => Array<Record<string, unknown>>
	 * }} AnnotatedText
	 */

	const BRIDGE_ID = 'harper-google-docs-main-world-bridge';
	const SYNC_INTERVAL_MS = 100;
	const EDITOR_SELECTOR = '.kix-appview-editor';
	const EDITOR_CONTAINER_SELECTOR = '.kix-appview-editor-container';
	const DOCS_EDITOR_SELECTOR = '#docs-editor';
	const CARET_SELECTOR = '.kix-cursor-caret';
	const TEXT_EVENT_IFRAME_SELECTOR = '.docs-texteventtarget-iframe';
	const LAYOUT_EPOCH_ATTR = 'data-harper-layout-epoch';
	const LAYOUT_REASON_ATTR = 'data-harper-layout-reason';
	const SELECTION_START_ATTR = 'data-harper-selection-start';
	const SELECTION_END_ATTR = 'data-harper-selection-end';
	const EVENT_TEXT_UPDATED = 'harper:gdocs:text-updated';
	const EVENT_LAYOUT_CHANGED = 'harper:gdocs:layout-changed';
	const EVENT_GET_RECTS = 'harper:gdocs:get-rects';
	const EVENT_REPLACE = 'harper:gdocs:replace';

	let isComputingRects = false;
	let layoutEpoch = 0;
	let layoutBumpPending = false;
	let lastCaretChoice = null;
	const CARET_DIRECTION_THRESHOLD = 20;
	const CARET_CHOICE_STALE_MS = 1500;

	/** @type {HTMLElement | null} */
	let bridge = document.getElementById(BRIDGE_ID);

	/** @returns {HTMLElement} */
	function ensureBridgeExists() {
		if (bridge) {
			return bridge;
		}

		const nextBridge = document.createElement('div');
		nextBridge.id = BRIDGE_ID;
		nextBridge.setAttribute('aria-hidden', 'true');
		nextBridge.style.display = 'none';
		document.documentElement.appendChild(nextBridge);
		bridge = nextBridge;

		return nextBridge;
	}

	ensureBridgeExists();

	/** @param {string} name
	 * @param {Record<string, unknown>} detail
	 * @returns {void}
	 */
	const emitEvent = (name, detail) => {
		try {
			document.dispatchEvent(new CustomEvent(name, { detail }));
		} catch {}
	};

	/**
	 * @param {string} reason
	 * @returns {void}
	 */
	const bumpLayoutEpoch = (reason) => {
		if (layoutBumpPending) return;
		layoutBumpPending = true;
		queueMicrotask(() => {
			layoutBumpPending = false;
			layoutEpoch += 1;
			const bridgeNode = ensureBridgeExists();
			bridgeNode.setAttribute(LAYOUT_EPOCH_ATTR, String(layoutEpoch));
			bridgeNode.setAttribute(LAYOUT_REASON_ATTR, String(reason));
			emitEvent(EVENT_LAYOUT_CHANGED, { layoutEpoch, reason });
		});
	};

	/** @returns {Promise<void>} */
	const syncText = async () => {
		try {
			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			/** @type {AnnotatedText | null | undefined} */
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.getText !== 'function') return;
			window.__harperGoogleDocsAnnotatedText = annotated;
			try {
				const selection = annotated.getSelection?.()?.[0];
				const endpoints = getSelectionEndpoints(selection);
				const bridgeNode = ensureBridgeExists();
				if (endpoints) {
					bridgeNode.setAttribute(SELECTION_START_ATTR, String(endpoints.start));
					bridgeNode.setAttribute(SELECTION_END_ATTR, String(endpoints.end));
				} else {
					bridgeNode.removeAttribute(SELECTION_START_ATTR);
					bridgeNode.removeAttribute(SELECTION_END_ATTR);
				}
			} catch {}
			const nextText = annotated.getText();
			const bridgeNode = ensureBridgeExists();
			if (bridgeNode.textContent !== nextText) {
				bridgeNode.textContent = nextText;
				emitEvent(EVENT_TEXT_UPDATED, { length: nextText.length });
			}
		} catch {}
	};

	/**
	 * @param {AnnotatedText} annotated
	 * @param {number} position
	 * @returns {Rect | null}
	 */
	const getCaretRect = (annotated, position) => {
		annotated.setSelection(position, position);
		const viewportWidth = window.innerWidth;
		const viewportHeight = window.innerHeight;
		const epsilon = 0.5;
		/** @type {Rect[]} */
		const carets = Array.from(document.querySelectorAll(CARET_SELECTOR))
			.map((caret) => {
				const rect = caret.getBoundingClientRect();
				const style = window.getComputedStyle(caret);
				const isFullyOnScreen =
					rect.left >= -epsilon &&
					rect.top >= -epsilon &&
					rect.right <= viewportWidth + epsilon &&
					rect.bottom <= viewportHeight + epsilon;
				if (
					!rect ||
					rect.width <= 0 ||
					rect.height <= 0 ||
					style.display === 'none' ||
					style.visibility === 'hidden' ||
					style.opacity === '0' ||
					!isFullyOnScreen
				) {
					return null;
				}
				return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
			})
			.filter((rect) => rect != null);
		if (carets.length === 0) return null;
		const inPage = carets.filter((rect) => rect.x > 100);
		const pool = inPage.length > 0 ? inPage : carets;
		const now = Date.now();
		if (!lastCaretChoice || now - lastCaretChoice.at > CARET_CHOICE_STALE_MS) {
			const seed = pool.reduce((best, rect) => (rect.x < best.x ? rect : best), pool[0]);
			lastCaretChoice = { rect: seed, position, at: now };
			return seed;
		}

		const deltaPos = position - lastCaretChoice.position;
		let chosen = null;

		if (deltaPos > CARET_DIRECTION_THRESHOLD) {
			const downward = pool.filter((rect) => rect.y >= lastCaretChoice.rect.y - 2);
			if (downward.length > 0) {
				chosen = downward.reduce((best, rect) => {
					if (rect.y > best.y + 1) return rect;
					if (Math.abs(rect.y - best.y) <= 1 && rect.x < best.x) return rect;
					return best;
				}, downward[0]);
			}
		} else if (deltaPos < -CARET_DIRECTION_THRESHOLD) {
			const upward = pool.filter((rect) => rect.y <= lastCaretChoice.rect.y + 2);
			if (upward.length > 0) {
				chosen = upward.reduce((best, rect) => {
					if (rect.y < best.y - 1) return rect;
					if (Math.abs(rect.y - best.y) <= 1 && rect.x < best.x) return rect;
					return best;
				}, upward[0]);
			}
		}

		if (!chosen) {
			chosen = pool.reduce((best, rect) => {
				const bestScore =
					Math.abs(best.y - lastCaretChoice.rect.y) * 4 + Math.abs(best.x - lastCaretChoice.rect.x);
				const rectScore =
					Math.abs(rect.y - lastCaretChoice.rect.y) * 4 + Math.abs(rect.x - lastCaretChoice.rect.x);
				return rectScore < bestScore ? rect : best;
			}, pool[0]);
		}

		lastCaretChoice = { rect: chosen, position, at: now };
		return chosen;
	};

	/**
	 * @param {unknown} value
	 * @returns {number | null}
	 */
	const asFiniteNumber = (value) => {
		const num = Number(value);
		return Number.isFinite(num) ? num : null;
	};

	/**
	 * @param {unknown} selection
	 * @returns {SelectionEndpoints | null}
	 */
	const getSelectionEndpoints = (selection) => {
		if (!selection || typeof selection !== 'object') {
			return null;
		}

		const candidates = [
			['anchor', 'focus'],
			['base', 'extent'],
			['start', 'end'],
		];

		for (const [a, b] of candidates) {
			const start = asFiniteNumber(selection[a]);
			const end = asFiniteNumber(selection[b]);
			if (start != null && end != null) {
				return { start, end };
			}
		}

		return null;
	};

	/**
	 * @param {AnnotatedText} annotated
	 * @param {SelectionEndpoints | null} selection
	 * @returns {void}
	 */
	const restoreSelection = (annotated, selection) => {
		if (!selection) return;
		try {
			annotated.setSelection(selection.start, selection.end);
		} catch {}
	};

	/**
	 * @param {number} start
	 * @param {number} end
	 * @param {SelectionEndpoints | null} selection
	 * @returns {boolean}
	 */
	const isSpanNearSelection = (start, end, selection) => {
		if (!selection) return false;
		const spanStart = Math.max(0, Math.min(start, end));
		const spanEnd = Math.max(spanStart, Math.max(start, end));
		const selStart = Math.min(selection.start, selection.end);
		const selEnd = Math.max(selection.start, selection.end);
		const maxDistance = 2000;

		if (spanStart <= selEnd && spanEnd >= selStart) {
			return true;
		}
		if (spanEnd < selStart) {
			return selStart - spanEnd <= maxDistance;
		}
		return spanStart - selEnd <= maxDistance;
	};

	/**
	 * @typedef {{
	 *   node: Window | Element,
	 *   left: number,
	 *   top: number
	 * }} ScrollSnapshot
	 */

	/**
	 * @returns {ScrollSnapshot[]}
	 */
	const snapshotScroll = () => {
		/** @type {ScrollSnapshot[]} */
		const snapshots = [{ node: window, left: window.scrollX, top: window.scrollY }];
		const selectors = [EDITOR_SELECTOR, EDITOR_CONTAINER_SELECTOR, DOCS_EDITOR_SELECTOR];
		for (const selector of selectors) {
			const node = document.querySelector(selector);
			if (!(node instanceof Element)) continue;
			if (snapshots.some((entry) => entry.node === node)) continue;
			snapshots.push({ node, left: node.scrollLeft, top: node.scrollTop });
		}
		return snapshots;
	};

	/**
	 * @param {ScrollSnapshot[]} snapshots
	 * @returns {void}
	 */
	const restoreScroll = (snapshots) => {
		for (const snap of snapshots) {
			if (snap.node === window) {
				if (window.scrollX !== snap.left || window.scrollY !== snap.top) {
					window.scrollTo(snap.left, snap.top);
				}
				continue;
			}
			if (!(snap.node instanceof Element)) continue;
			if (snap.node.scrollLeft !== snap.left) {
				snap.node.scrollLeft = snap.left;
			}
			if (snap.node.scrollTop !== snap.top) {
				snap.node.scrollTop = snap.top;
			}
		}
	};

	/**
	 * @template T
	 * @param {() => T} fn
	 * @returns {T}
	 */
	const withSuppressedScrolling = (fn) => {
		/** @type {Array<() => void>} */
		const restorers = [];
		const noop = () => {};
		const tryOverride = (obj, key) => {
			if (!obj) return;
			const original = obj[key];
			if (typeof original !== 'function') return;
			try {
				obj[key] = noop;
				restorers.push(() => {
					try {
						obj[key] = original;
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
			for (let i = restorers.length - 1; i >= 0; i -= 1) {
				restorers[i]();
			}
		}
	};

	/** @param {Event} event */
	document.addEventListener(EVENT_GET_RECTS, (event) => {
		try {
			/** @type {{ requestId?: string, start?: number, end?: number }} */
			const detail = /** @type {CustomEvent} */ (event).detail || {};
			const requestId = String(detail.requestId || '');
			if (!requestId) return;
			const start = Number(detail.start);
			const end = Number(detail.end);
			/** @type {AnnotatedText | undefined} */
			const annotated = window.__harperGoogleDocsAnnotatedText;
			if (!annotated || typeof annotated.setSelection !== 'function') return;

			const currentSelection = annotated.getSelection?.()?.[0];
			const previousSelection = getSelectionEndpoints(currentSelection);
			if (!isSpanNearSelection(start, end, previousSelection)) {
				ensureBridgeExists().setAttribute(`data-harper-rects-${requestId}`, '[]');
				return;
			}

			/** @type {Rect[]} */
			const rects = [];
			isComputingRects = true;
			const scrollSnapshot = snapshotScroll();
			try {
				const spanStart = Math.max(0, Math.min(start, end));
				const spanEnd = Math.max(spanStart, end);
				const { startRect, endRect } = withSuppressedScrolling(() => {
					return {
						startRect: getCaretRect(annotated, spanStart),
						endRect: getCaretRect(annotated, spanEnd),
					};
				});

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
			} finally {
				isComputingRects = false;
				restoreSelection(annotated, previousSelection);
				restoreScroll(scrollSnapshot);
				requestAnimationFrame(() => restoreScroll(scrollSnapshot));
			}

			ensureBridgeExists().setAttribute(`data-harper-rects-${requestId}`, JSON.stringify(rects));
		} catch {}
	});

	document.addEventListener(EVENT_REPLACE, async (event) => {
		try {
			/** @type {{ start?: number, end?: number, replacementText?: string, expectedText?: string, beforeContext?: string, afterContext?: string }} */
			const detail = /** @type {CustomEvent} */ (event).detail || {};
			let start = Number(detail.start);
			let end = Number(detail.end);
			const replacementText = String(detail.replacementText ?? '');
			const expectedText = String(detail.expectedText ?? '');
			const beforeContext = String(detail.beforeContext ?? '');
			const afterContext = String(detail.afterContext ?? '');
			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			/** @type {AnnotatedText | null | undefined} */
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.setSelection !== 'function') return;

			if (expectedText) {
				const currentText = annotated.getText?.();
				if (typeof currentText === 'string') {
					const spanLength = Math.max(0, end - start);
					const normalizeStart = Math.max(0, Math.min(start, currentText.length));
					const normalizeEnd = Math.max(normalizeStart, Math.min(end, currentText.length));
					const direct = currentText.slice(normalizeStart, normalizeEnd);
					if (direct !== expectedText) {
						const resolveByOffsetWindow = () => {
							for (let delta = -12; delta <= 12; delta += 1) {
								const candidateStart = normalizeStart + delta;
								if (candidateStart < 0) continue;
								const candidateEnd = candidateStart + spanLength;
								if (candidateEnd > currentText.length) continue;
								if (currentText.slice(candidateStart, candidateEnd) === expectedText) {
									return { start: candidateStart, end: candidateEnd };
								}
							}
							return null;
						};

						const resolveByContext = () => {
							const hits = [];
							let cursor = 0;
							while (cursor <= currentText.length) {
								const index = currentText.indexOf(expectedText, cursor);
								if (index < 0) break;
								const indexEnd = index + expectedText.length;
								const beforeTail = beforeContext ? beforeContext.slice(-16) : '';
								const afterHead = afterContext ? afterContext.slice(0, 16) : '';
								let score = 0;
								if (beforeTail && currentText.slice(Math.max(0, index - beforeTail.length), index) === beforeTail) {
									score += 2;
								}
								if (afterHead && currentText.slice(indexEnd, Math.min(currentText.length, indexEnd + afterHead.length)) === afterHead) {
									score += 2;
								}
								score -= Math.abs(index - normalizeStart) / 1000;
								hits.push({ start: index, end: indexEnd, score });
								cursor = index + 1;
							}

							if (hits.length === 0) return null;
							hits.sort((a, b) => b.score - a.score);
							return { start: hits[0].start, end: hits[0].end };
						};

						const resolved = resolveByOffsetWindow() ?? resolveByContext();
						if (resolved) {
							start = resolved.start;
							end = resolved.end;
						}
					}
				}
			}

			annotated.setSelection(start, end);
			const iframe = document.querySelector(TEXT_EVENT_IFRAME_SELECTOR);
			const target = iframe?.contentDocument?.activeElement;
			if (!target) return;
			const dt = new DataTransfer();
			dt.setData('text/plain', replacementText);
			const pasteEvent = new ClipboardEvent('paste', {
				clipboardData: dt,
				cancelable: true,
				bubbles: true,
			});
			target.dispatchEvent(pasteEvent);
			setTimeout(syncText, 0);
		} catch {}
	});

	window.addEventListener('resize', () => bumpLayoutEpoch('resize'));

	const observeLayout = () => {
		const editor = document.querySelector(EDITOR_SELECTOR);
		if (!(editor instanceof HTMLElement)) {
			setTimeout(observeLayout, 250);
			return;
		}

		bumpLayoutEpoch('init');
		const observer = new MutationObserver((mutations) => {
			for (const mutation of mutations) {
				if (mutation.type === 'childList' || mutation.type === 'attributes') {
					bumpLayoutEpoch('mutation');
					break;
				}
			}
		});
		observer.observe(editor, {
			subtree: true,
			childList: true,
			attributes: true,
			attributeFilter: ['style', 'class'],
		});
	};

	observeLayout();
	syncText();
	setInterval(syncText, SYNC_INTERVAL_MS);
})();

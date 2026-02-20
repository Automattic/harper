(() => {
	/**
	 * @typedef {{ x: number, y: number, width: number, height: number }} Rect
	 */

	/**
	 * @typedef {{ start: number, end: number }} SelectionEndpoints
	 */

	/**
	 * @typedef {{ type: 'window', x: number, y: number }} WindowScrollEntry
	 */

	/**
	 * @typedef {{ type: 'element', el: HTMLElement, top: number, left: number }} ElementScrollEntry
	 */

	/**
	 * @typedef {WindowScrollEntry | ElementScrollEntry} ScrollStateEntry
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
	const USER_SCROLL_INTENT_WINDOW_MS = 150;
	const EDITOR_SELECTOR = '.kix-appview-editor';
	const EDITOR_CONTAINER_SELECTOR = '.kix-appview-editor-container';
	const DOCS_EDITOR_SELECTOR = '#docs-editor';
	const CARET_SELECTOR = '.kix-cursor-caret';
	const TEXT_EVENT_IFRAME_SELECTOR = '.docs-texteventtarget-iframe';
	const LAYOUT_EPOCH_ATTR = 'data-harper-layout-epoch';
	const LAYOUT_REASON_ATTR = 'data-harper-layout-reason';
	const EVENT_TEXT_UPDATED = 'harper:gdocs:text-updated';
	const EVENT_LAYOUT_CHANGED = 'harper:gdocs:layout-changed';
	const EVENT_GET_RECTS = 'harper:gdocs:get-rects';
	const EVENT_REPLACE = 'harper:gdocs:replace';

	let isComputingRects = false;
	let lastKnownEditorScrollTop = -1;
	let lastUserScrollAt = 0;
	let userInteractionEpoch = 0;
	let layoutEpoch = 0;
	let layoutBumpPending = false;

	/** @type {HTMLElement | null} */
	let bridge = document.getElementById(BRIDGE_ID);

	/** Makes sure the bridge exists, creating it if it doesn't.
	 * Returns said bridge.
	 *
	 * @returns {HTMLElement} */
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

	/** Dispatch an event to communicate with the user script.
	 *
	 * @param {string} name
	 * @param {Record<string, unknown>} detail
	 * @returns {void}
	 */
	const emitEvent = (name, detail) => {
		try {
			document.dispatchEvent(new CustomEvent(name, { detail }));
		} catch {
			// Ignore event emission failures.
		}
	};

	const markUserScrollIntent = () => {
		lastUserScrollAt = Date.now();
		userInteractionEpoch += 1;
	};

	/** @returns {boolean} */
	const isUserActivelyScrolling = () =>
		Date.now() - lastUserScrollAt < USER_SCROLL_INTENT_WINDOW_MS;

	/** @param {KeyboardEvent} event */
	const isScrollLayoutKey = (event) =>
		event.key === 'PageDown' ||
		event.key === 'PageUp' ||
		event.key === 'Home' ||
		event.key === 'End';

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
			const editor = document.querySelector(EDITOR_SELECTOR);
			if (editor instanceof HTMLElement && lastKnownEditorScrollTop < 0) {
				lastKnownEditorScrollTop = editor.scrollTop;
			}

			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			/** @type {AnnotatedText | null | undefined} */
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.getText !== 'function') return;
			window.__harperGoogleDocsAnnotatedText = annotated;
			const nextText = annotated.getText();
			const bridgeNode = ensureBridgeExists();
			if (bridgeNode.textContent !== nextText) {
				bridgeNode.textContent = nextText;
				emitEvent(EVENT_TEXT_UPDATED, { length: nextText.length });
			}
		} catch {
			// Ignore intermittent Docs internal errors.
		}
	};

	/** @returns {ScrollStateEntry[]} */
	const getScrollState = () => {
		/** @type {ScrollStateEntry[]} */
		const state = [];
		state.push({ type: 'window', x: window.scrollX, y: window.scrollY });

		const candidates = new Set();
		/** @param {Element | null} node */
		const addCandidate = (node) => {
			if (node instanceof HTMLElement) {
				candidates.add(node);
			}
		};
		/** @param {Element | null} node */
		const addElementAndAncestors = (node) => {
			if (!(node instanceof HTMLElement)) return;
			addCandidate(node);
			let parent = node.parentElement;
			while (parent) {
				addCandidate(parent);
				parent = parent.parentElement;
			}
		};

		addElementAndAncestors(document.querySelector(EDITOR_SELECTOR));
		addElementAndAncestors(document.querySelector(EDITOR_CONTAINER_SELECTOR));
		addElementAndAncestors(document.querySelector(DOCS_EDITOR_SELECTOR));
		addElementAndAncestors(document.activeElement);

		for (const node of candidates) {
			if (node.scrollTop !== 0 || node.scrollLeft !== 0 || node.matches(EDITOR_SELECTOR)) {
				state.push({ type: 'element', el: node, top: node.scrollTop, left: node.scrollLeft });
			}
		}

		return state;
	};

	/**
	 * @param {ScrollStateEntry[]} state
	 * @returns {void}
	 */
	const restoreScrollState = (state) => {
		for (const entry of state) {
			if (entry.type === 'window') {
				window.scrollTo(entry.x, entry.y);
				continue;
			}

			if (!entry.el || !entry.el.isConnected) continue;
			entry.el.scrollTop = entry.top;
			entry.el.scrollLeft = entry.left;
		}
	};

	/**
	 * @param {ScrollStateEntry[]} state
	 * @returns {boolean}
	 */
	const didScrollStateChange = (state) => {
		for (const entry of state) {
			if (entry.type === 'window') {
				if (window.scrollX !== entry.x || window.scrollY !== entry.y) {
					return true;
				}
				continue;
			}

			if (!entry.el || !entry.el.isConnected) continue;
			if (entry.el.scrollTop !== entry.top || entry.el.scrollLeft !== entry.left) {
				return true;
			}
		}

		return false;
	};

	/**
	 * @param {AnnotatedText} annotated
	 * @param {number} position
	 * @returns {Rect | null}
	 */
	const getCaretRect = (annotated, position) => {
		annotated.setSelection(position, position);
		/** @type {Rect[]} */
		const carets = Array.from(document.querySelectorAll(CARET_SELECTOR))
			.map((caret) => {
				const rect = caret.getBoundingClientRect();
				const style = window.getComputedStyle(caret);
				if (
					!rect ||
					rect.width <= 0 ||
					rect.height <= 0 ||
					style.display === 'none' ||
					style.visibility === 'hidden' ||
					style.opacity === '0'
				) {
					return null;
				}
				return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
			})
			.filter((rect) => rect != null);
		if (carets.length === 0) return null;
		const inPage = carets.filter((rect) => rect.x > 100);
		const pool = inPage.length > 0 ? inPage : carets;
		return pool.reduce((best, rect) => (rect.x < best.x ? rect : best), pool[0]);
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
		} catch {
			// Ignore selection restore failures.
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
			const interactionEpochAtStart = userInteractionEpoch;

			const scrollState = getScrollState();
			const currentSelection = annotated.getSelection?.()?.[0];
			const previousSelection = getSelectionEndpoints(currentSelection);

			/** @type {Rect[]} */
			const rects = [];
			isComputingRects = true;
			try {
				const spanStart = Math.max(0, Math.min(start, end));
				const spanEnd = Math.max(spanStart, end);
				const startRect = getCaretRect(annotated, spanStart);
				const endRect = getCaretRect(annotated, spanEnd);

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

				// Restore viewport only if bridge operations changed it and the user
				// has not interacted while we were computing.
				if (
					didScrollStateChange(scrollState) &&
					!isUserActivelyScrolling() &&
					interactionEpochAtStart === userInteractionEpoch
				) {
					restoreScrollState(scrollState);
				}
			}

			ensureBridgeExists().setAttribute(`data-harper-rects-${requestId}`, JSON.stringify(rects));
		} catch {
			// No-op.
		}
	});

	/** @param {Event} event */
	document.addEventListener(EVENT_REPLACE, async (event) => {
		try {
			/** @type {{ start?: number, end?: number, replacementText?: string }} */
			const detail = /** @type {CustomEvent} */ (event).detail || {};
			const start = Number(detail.start);
			const end = Number(detail.end);
			const replacementText = String(detail.replacementText ?? '');
			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			/** @type {AnnotatedText | null | undefined} */
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.setSelection !== 'function') return;
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
		} catch {
			// No-op.
		}
	});

	document.addEventListener(
		'scroll',
		() => {
			markUserScrollIntent();
			const editor = document.querySelector(EDITOR_SELECTOR);
			if (!(editor instanceof HTMLElement)) return;

			if (lastKnownEditorScrollTop < 0) {
				lastKnownEditorScrollTop = editor.scrollTop;
				return;
			}

			if (editor.scrollTop === lastKnownEditorScrollTop) {
				return;
			}

			lastKnownEditorScrollTop = editor.scrollTop;

			if (!isComputingRects) {
				bumpLayoutEpoch('scroll');
			}
		},
		true,
	);

	document.addEventListener(
		'wheel',
		/** @param {WheelEvent} event */
		(event) => {
			markUserScrollIntent();
			const target = event.target;
			if (
				target instanceof HTMLElement &&
				(target.classList.contains('kix-appview-editor') ||
					target.closest(EDITOR_SELECTOR) != null ||
					target.id === 'docs-editor')
			) {
				bumpLayoutEpoch('wheel');
			}
		},
		{ capture: true, passive: true },
	);

	document.addEventListener(
		'keydown',
		/** @param {KeyboardEvent} event */
		(event) => {
			if (isScrollLayoutKey(event)) {
				markUserScrollIntent();
				bumpLayoutEpoch('key-scroll');
			}
		},
		true,
	);

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

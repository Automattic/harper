/*
 * Google Docs main-world bridge.
 *
 * Why this file exists:
 * - The extension content script runs in an isolated world, while Google Docs keeps its editor
 *   internals (for example `_docs_annotate_getAnnotatedText`) in the page's main world.
 * - We need access to those Docs internals to:
 *   1) read the canonical document text,
 *   2) compute approximate on-screen rects for lint spans, and
 *   3) perform replacements in a way Docs accepts.
 * - To bridge that isolation boundary, this script is injected into the main world and
 *   communicates with the content script through:
 *   - a hidden DOM node (`#harper-google-docs-main-world-bridge`) used as a shared state surface,
 *   - custom DOM events for commands and notifications.
 *
 * High-level flow:
 * 1) Text sync:
 *    - Polls Docs (`_docs_annotate_getAnnotatedText`) at a short interval.
 *    - Writes the current plain text into the hidden bridge node's `textContent`.
 *    - Emits `harper:gdocs:text-updated` when text changes so the content script can re-lint.
 *
 * 2) Layout invalidation:
 *    - Monitors scroll/wheel/key-scroll/resize/editor mutations.
 *    - Bumps a monotonically increasing `layoutEpoch` and mirrors it on the bridge node
 *      (`data-harper-layout-epoch`).
 *    - Emits `harper:gdocs:layout-changed` so highlight positioning can be refreshed.
 *    - Uses microtask coalescing to avoid flooding updates when many layout signals occur together.
 *
 * 3) Rect requests:
 *    - Listens for `harper:gdocs:get-rects` with `{ requestId, start, end }`.
 *    - Temporarily moves Docs selection to span boundaries and inspects visible caret rects
 *      (`.kix-cursor-caret`) as a practical proxy for span geometry.
 *    - Restores prior selection and scroll state to minimize user-visible side effects.
 *    - Returns computed rects via `data-harper-rects-${requestId}` on the bridge node.
 *
 * 4) Replacements:
 *    - Listens for `harper:gdocs:replace` with `{ start, end, replacementText }`.
 *    - Selects the span in Docs internals and dispatches a synthetic paste event into Docs'
 *      text input target iframe.
 *    - Triggers a deferred text sync so the content script sees the post-edit text.
 *
 * Reliability constraints and design choices:
 * - Google Docs can throw intermittent internal errors while the editor is updating. Most bridge
 *   operations are guarded with `try/catch` and fail soft so the extension keeps running.
 * - DOM/state writes are intentionally minimal (only write when changed) to reduce churn.
 * - The bridge node is kept hidden and inert (`aria-hidden`, `display:none`) and is only used for
 *   cross-world signaling/state handoff.
 * - This file is plain JS and intentionally self-contained because it is injected directly into the
 *   page, outside the extension module graph.
 */

(() => {
	const BRIDGE_ID = 'harper-google-docs-main-world-bridge';
	const SYNC_INTERVAL_MS = 100;
	const EDITOR_SELECTOR = '.kix-appview-editor';
	const EDITOR_CONTAINER_SELECTOR = '.kix-appview-editor-container';
	const DOCS_EDITOR_SELECTOR = '#docs-editor';
	const CARET_SELECTOR = '.kix-cursor-caret';
	const TEXT_EVENT_IFRAME_SELECTOR = '.docs-texteventtarget-iframe';
	const EVENT_TEXT_UPDATED = 'harper:gdocs:text-updated';
	const EVENT_LAYOUT_CHANGED = 'harper:gdocs:layout-changed';
	const EVENT_GET_RECTS = 'harper:gdocs:get-rects';
	const EVENT_REPLACE = 'harper:gdocs:replace';

	let isComputingRects = false;
	let lastKnownEditorScrollTop = -1;
	let layoutEpoch = 0;
	let layoutBumpPending = false;

	let bridge = document.getElementById(BRIDGE_ID);

	function ensureBridge() {
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

	ensureBridge();

	const emitEvent = (name, detail) => {
		try {
			document.dispatchEvent(new CustomEvent(name, { detail }));
		} catch {
			// Ignore event emission failures.
		}
	};

	const bumpLayoutEpoch = (reason) => {
		if (layoutBumpPending) return;
		layoutBumpPending = true;
		queueMicrotask(() => {
			layoutBumpPending = false;
			layoutEpoch += 1;
			ensureBridge().setAttribute('data-harper-layout-epoch', String(layoutEpoch));
			emitEvent(EVENT_LAYOUT_CHANGED, { layoutEpoch, reason });
		});
	};

	const syncText = async () => {
		try {
			const editor = document.querySelector(EDITOR_SELECTOR);
			if (editor instanceof HTMLElement && lastKnownEditorScrollTop < 0) {
				lastKnownEditorScrollTop = editor.scrollTop;
			}

			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.getText !== 'function') return;
			window.__harperGoogleDocsAnnotatedText = annotated;
			const nextText = annotated.getText();
			const bridgeNode = ensureBridge();
			if (bridgeNode.textContent !== nextText) {
				bridgeNode.textContent = nextText;
				emitEvent(EVENT_TEXT_UPDATED, { length: nextText.length });
			}
		} catch {
			// Ignore intermittent Docs internal errors.
		}
	};

	const getScrollState = () => {
		const state = [];
		state.push({ type: 'window', x: window.scrollX, y: window.scrollY });

		const candidates = new Set();
		const addCandidate = (node) => {
			if (node instanceof HTMLElement) {
				candidates.add(node);
			}
		};
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

	const getCaretRect = (annotated, position) => {
		annotated.setSelection(position, position);
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

	document.addEventListener(EVENT_GET_RECTS, (event) => {
		try {
			const detail = event.detail || {};
			const requestId = String(detail.requestId || '');
			if (!requestId) return;
			const start = Number(detail.start);
			const end = Number(detail.end);
			const annotated = window.__harperGoogleDocsAnnotatedText;
			if (!annotated || typeof annotated.setSelection !== 'function') return;

			const scrollState = getScrollState();
			const currentSelection = annotated.getSelection?.()?.[0] || null;
			const previousSelection =
				currentSelection &&
				Number.isFinite(Number(currentSelection.start)) &&
				Number.isFinite(Number(currentSelection.end))
					? {
							start: Number(currentSelection.start),
							end: Number(currentSelection.end),
						}
					: null;

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
				if (previousSelection) {
					try {
						annotated.setSelection(previousSelection.start, previousSelection.end);
					} catch {
						// Ignore selection restore failures.
					}
				}
				if (!didScrollStateChange(scrollState)) {
					restoreScrollState(scrollState);
				}
			}

			ensureBridge().setAttribute(`data-harper-rects-${requestId}`, JSON.stringify(rects));
		} catch {
			// No-op.
		}
	});

	document.addEventListener(EVENT_REPLACE, async (event) => {
		try {
			const detail = event.detail || {};
			const start = Number(detail.start);
			const end = Number(detail.end);
			const replacementText = String(detail.replacementText ?? '');
			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
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
		(event) => {
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
		(event) => {
			if (
				event.key === 'PageDown' ||
				event.key === 'PageUp' ||
				event.key === 'Home' ||
				event.key === 'End'
			) {
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

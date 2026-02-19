(() => {
	const BRIDGE_ID = 'harper-google-docs-main-world-bridge';
	const USER_SCROLL_COOLDOWN_MS = 60000;
	const USER_INTENT_WINDOW_MS = 1200;
	const SYNC_INTERVAL_MS = 100;
	let lastUserScrollAt = 0;
	let hasUserScrolled = false;
	let isComputingRects = false;
	let lastKnownEditorScrollTop = -1;
	let lastUserIntentAt = 0;
	let layoutEpoch = 0;
	let layoutBumpPending = false;

	let bridge = document.getElementById(BRIDGE_ID);
	if (!bridge) {
		bridge = document.createElement('div');
		bridge.id = BRIDGE_ID;
		bridge.setAttribute('aria-hidden', 'true');
		bridge.style.display = 'none';
		document.documentElement.appendChild(bridge);
	}

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
			bridge.setAttribute('data-harper-layout-epoch', String(layoutEpoch));
			emitEvent('harper:gdocs:layout-changed', { layoutEpoch, reason });
		});
	};

	const noteUserIntent = () => {
		lastUserIntentAt = Date.now();
	};

	const syncText = async () => {
		try {
			const editor = document.querySelector('.kix-appview-editor');
			if (editor instanceof HTMLElement && lastKnownEditorScrollTop < 0) {
				lastKnownEditorScrollTop = editor.scrollTop;
			}

			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.getText !== 'function') return;
			window.__harperGoogleDocsAnnotatedText = annotated;
			const nextText = annotated.getText();
			if (bridge.textContent !== nextText) {
				bridge.textContent = nextText;
				emitEvent('harper:gdocs:text-updated', { length: nextText.length });
			}
		} catch {
			// Ignore intermittent Docs internal errors.
		}
	};

	const getScrollState = () => {
		const state = [];
		state.push({ type: 'window', x: window.scrollX, y: window.scrollY });

		const keep = new Set(
			Array.from(
				document.querySelectorAll('.kix-appview-editor, .kix-appview-editor-container, #docs-editor'),
			),
		);

		for (const el of document.querySelectorAll('*')) {
			const node = el;
			if (!(node instanceof HTMLElement)) continue;
			if (node.scrollTop !== 0 || node.scrollLeft !== 0 || keep.has(node)) {
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

	const getCaretRect = (annotated, position) => {
		annotated.setSelection(position, position);
		const caret = document.querySelector('.kix-cursor-caret');
		if (!caret) return null;
		const rect = caret.getBoundingClientRect();
		if (!rect || rect.width <= 0 || rect.height <= 0) return null;
		return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
	};

	document.addEventListener('harper:gdocs:get-rects', (event) => {
		try {
			const detail = event.detail || {};
			const requestId = String(detail.requestId || '');
			if (!requestId) return;
			const start = Number(detail.start);
			const end = Number(detail.end);
			const annotated = window.__harperGoogleDocsAnnotatedText;
			if (!annotated || typeof annotated.setSelection !== 'function') return;

			if (hasUserScrolled || Date.now() - lastUserScrollAt < USER_SCROLL_COOLDOWN_MS) {
				bridge.setAttribute(`data-harper-rects-${requestId}`, JSON.stringify([]));
				return;
			}

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
				if (!hasUserScrolled) {
					restoreScrollState(scrollState);
				}
			}

			bridge.setAttribute(`data-harper-rects-${requestId}`, JSON.stringify(rects));
		} catch {
			// No-op.
		}
	});

	document.addEventListener('harper:gdocs:replace', async (event) => {
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
			const iframe = document.querySelector('.docs-texteventtarget-iframe');
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
			const editor = document.querySelector('.kix-appview-editor');
			if (!(editor instanceof HTMLElement)) return;

			if (lastKnownEditorScrollTop < 0) {
				lastKnownEditorScrollTop = editor.scrollTop;
				return;
			}

			if (editor.scrollTop === lastKnownEditorScrollTop) {
				return;
			}

			lastKnownEditorScrollTop = editor.scrollTop;

			if (!isComputingRects && Date.now() - lastUserIntentAt < USER_INTENT_WINDOW_MS) {
				lastUserScrollAt = Date.now();
				hasUserScrolled = true;
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
					target.closest('.kix-appview-editor') != null ||
					target.id === 'docs-editor')
			) {
				noteUserIntent();
				lastUserScrollAt = Date.now();
				hasUserScrolled = true;
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
				noteUserIntent();
				lastUserScrollAt = Date.now();
				hasUserScrolled = true;
			}
		},
		true,
	);

	document.addEventListener(
		'pointerdown',
		(event) => {
			const target = event.target;
			if (
				target instanceof HTMLElement &&
				(target.classList.contains('kix-appview-editor') ||
					target.closest('.kix-appview-editor') != null ||
					target.id === 'docs-editor')
			) {
				noteUserIntent();
			}
		},
		true,
	);

	window.addEventListener('resize', () => bumpLayoutEpoch('resize'));

	const observeLayout = () => {
		const editor = document.querySelector('.kix-appview-editor');
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

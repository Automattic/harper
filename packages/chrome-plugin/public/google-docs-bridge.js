(() => {
	const BRIDGE_ID = 'harper-google-docs-main-world-bridge';

	let bridge = document.getElementById(BRIDGE_ID);
	if (!bridge) {
		bridge = document.createElement('div');
		bridge.id = BRIDGE_ID;
		bridge.setAttribute('aria-hidden', 'true');
		bridge.style.display = 'none';
		document.documentElement.appendChild(bridge);
	}

	const syncText = async () => {
		try {
			const getAnnotatedText = window._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			const annotated = await getAnnotatedText();
			if (!annotated || typeof annotated.getText !== 'function') return;
			window.__harperGoogleDocsAnnotatedText = annotated;
			bridge.textContent = annotated.getText();
		} catch {
			// Ignore intermittent Docs internal errors.
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

			const previousSelection = annotated.getSelection?.()?.[0] || null;
			const spanStart = Math.max(0, Math.min(start, end));
			const spanEnd = Math.max(spanStart, end);
			const startRect = getCaretRect(annotated, spanStart);
			const endRect = getCaretRect(annotated, spanEnd);

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

			if (previousSelection) {
				const prevStart = Number(previousSelection.start);
				const prevEnd = Number(previousSelection.end);
				if (prevStart === prevEnd) {
					annotated.setSelection(prevStart, prevStart);
				} else {
					annotated.setSelection(prevStart, prevEnd + 1);
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

	syncText();
	setInterval(syncText, 300);
})();

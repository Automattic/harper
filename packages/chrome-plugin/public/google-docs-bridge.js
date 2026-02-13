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
			bridge.textContent = annotated.getText();
		} catch {
			// Ignore intermittent Docs internal errors.
		}
	};

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

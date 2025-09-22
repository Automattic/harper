(() => {
	// Force the fallback path by removing CSS Highlight API support before scripts run.
	try {
		const originalSupports = CSS.supports.bind(CSS);
		Object.defineProperty(CSS, 'supports', {
			configurable: true,
			value(selector, ...rest) {
				if (typeof selector === 'string' && selector.startsWith('selector(::highlight(')) {
					return false;
				}
				return originalSupports(selector, ...rest);
			},
		});
	} catch {}

	try {
		delete CSS.highlights;
	} catch {
		try {
			Object.defineProperty(CSS, 'highlights', {
				configurable: true,
				get() {
					return undefined;
				},
			});
		} catch {}
	}

	try {
		delete window.Highlight;
	} catch {
		try {
			Object.defineProperty(window, 'Highlight', {
				configurable: true,
				get() {
					return undefined;
				},
			});
		} catch {}
	}
})();

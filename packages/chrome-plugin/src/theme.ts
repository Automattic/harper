const QUERY = '(prefers-color-scheme: dark)';

function applyDarkTheme(shouldUseDark: boolean) {
	const root = document.documentElement;
	const body = document.body;

	root.classList.toggle('dark', shouldUseDark);
	body?.classList.toggle('dark', shouldUseDark);
}

export function setupTheme() {
	if (typeof window === 'undefined' || typeof document === 'undefined') {
		return;
	}

	const mediaQuery = window.matchMedia(QUERY);

	applyDarkTheme(mediaQuery.matches);

	const listener = (event: MediaQueryListEvent) => {
		applyDarkTheme(event.matches);
	};

	if ('addEventListener' in mediaQuery) {
		mediaQuery.addEventListener('change', listener);
	} else {
		mediaQuery.addListener(listener);
	}
}

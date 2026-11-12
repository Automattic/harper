import { type Readable, writable } from 'svelte/store';

/** Persist only explicit choices. An absent or invalid value follows the OS theme. */
export const THEME_STORAGE_KEY = 'HARPER_THEME';
/** Context is provided by the root layout, including on marketing routes. */
export const THEME_CONTEXT = Symbol('Harper theme');

export type Theme = 'light' | 'dark';

/** An absent or invalid saved preference follows the OS theme. */
export function resolveThemePreference(
	stored: string | null,
	systemDark: boolean,
): { theme: Theme; preference: Theme | null } {
	const preference = stored === 'light' || stored === 'dark' ? stored : null;
	return { preference, theme: preference ?? (systemDark ? 'dark' : 'light') };
}

function readPreference(key: string): string | null {
	try {
		return localStorage.getItem(key);
	} catch {
		return null;
	}
}

/**
 * A root-owned theme store. Construction is SSR-safe and state belongs to each instance.
 * start() owns browser listeners and returns idempotent cleanup.
 */
export default class ThemeController implements Readable<Theme> {
	private readonly state = writable<Theme>('light');
	private preference: Theme | null = null;
	private stop: (() => void) | undefined;

	// Keep subscribe callable without a receiver for Svelte's store contract.
	readonly subscribe = this.state.subscribe;

	private apply(theme: Theme) {
		this.state.set(theme);
		if (typeof document !== 'undefined') {
			document.documentElement.classList.toggle('dark', theme === 'dark');
			document.documentElement.style.colorScheme = theme;
		}
	}

	/** Apply immediately, retaining the choice in this tab if storage writes fail. */
	setTheme(theme: Theme): void {
		this.preference = theme;
		this.apply(theme);
		try {
			localStorage.setItem(THEME_STORAGE_KEY, theme);
		} catch {
			// Keep the in-memory choice even when persistence is unavailable.
		}
	}

	/** Initialize browser state once per active lifecycle; cleanup permits a later restart. */
	start(): () => void {
		if (typeof window === 'undefined') return () => {};
		if (this.stop) return this.stop;
		const media = window.matchMedia?.('(prefers-color-scheme: dark)');
		const stored = readPreference(THEME_STORAGE_KEY);
		const resolved = resolveThemePreference(stored, media?.matches ?? false);
		this.preference = resolved.preference;
		this.apply(resolved.theme);

		const onSystemChange = () => {
			if (!this.preference) this.apply(media?.matches ? 'dark' : 'light');
		};
		const onStorage = (event: StorageEvent) => {
			if (event.key !== THEME_STORAGE_KEY && event.key !== null) return;
			try {
				if (event.storageArea !== localStorage) return;
			} catch {
				return;
			}
			// Do not write back across tabs.
			const resolved = resolveThemePreference(event.newValue, media?.matches ?? false);
			this.preference = resolved.preference;
			this.apply(resolved.theme);
		};
		media?.addEventListener('change', onSystemChange);
		window.addEventListener('storage', onStorage);
		let stopped = false;
		this.stop = () => {
			if (stopped) return;
			stopped = true;
			media?.removeEventListener('change', onSystemChange);
			window.removeEventListener('storage', onStorage);
			this.stop = undefined;
		};
		return this.stop;
	}
}

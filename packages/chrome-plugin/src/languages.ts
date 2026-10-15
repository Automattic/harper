import type { Dialect, DialectInfo } from 'harper.js';

/** How many languages the keyboard shortcut cycles through at most. */
export const MAX_LANGUAGE_CYCLE = 4;

export function dialectInfo(catalog: DialectInfo[], dialect: Dialect): DialectInfo | undefined {
	return catalog.find((info) => info.dialect === dialect);
}

/** A human readable name such as "English (British)" or "Slovenčina". */
export function languageLabel(info: DialectInfo | undefined): string {
	if (info == null) {
		return '?';
	}

	return info.language === info.region ? info.language : `${info.language} (${info.region})`;
}

/** The flag emoji of a two-letter country code, or nothing for other codes. */
export function codeFlag(code: string): string {
	if (!/^[A-Z]{2}$/.test(code)) {
		return '';
	}

	return String.fromCodePoint(...[...code].map((letter) => 0x1f1e6 + letter.charCodeAt(0) - 65));
}

/** The catalog's dialects grouped by language, in catalog order. */
export function groupByLanguage(catalog: DialectInfo[]): [string, DialectInfo[]][] {
	const groups = new Map<string, DialectInfo[]>();
	for (const info of catalog) {
		groups.set(info.language, [...(groups.get(info.language) ?? []), info]);
	}
	return [...groups];
}

/**
 * Turn whatever is stored into a usable cycle: dialects of this build only, no duplicates, at
 * most {@link MAX_LANGUAGE_CYCLE}, and never empty.
 */
export function normalizeLanguageCycle(
	stored: unknown,
	fallback: Dialect,
	catalog: DialectInfo[],
): Dialect[] {
	const cycle: Dialect[] = [];

	if (Array.isArray(stored)) {
		for (const value of stored) {
			if (dialectInfo(catalog, value) != null && !cycle.includes(value)) {
				cycle.push(value);
			}
		}
	}

	if (cycle.length === 0) {
		cycle.push(fallback);
	}

	return cycle.slice(0, MAX_LANGUAGE_CYCLE);
}

/** The language after `current` in the cycle, wrapping around to the first one. */
export function nextLanguage(cycle: Dialect[], current: Dialect): Dialect {
	const index = cycle.indexOf(current);
	return cycle[(index + 1) % cycle.length];
}

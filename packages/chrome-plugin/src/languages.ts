import type { DialectInfo } from 'harper.js';

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

import { Dialect } from 'harper.js';

/** Detect English dialect from browser language settings */
export function detectBrowserDialect(): Dialect {
	// Try chrome.i18n API first
	if (chrome.i18n?.getUILanguage) {
		const locale = chrome.i18n.getUILanguage();
		return localeToDialect(locale);
	}

	// Fallback to navigator.language
	const lang = navigator.language || navigator.languages?.[0] || 'en-US';
	return localeToDialect(lang);
}

/** Map locale string to Dialect */
function localeToDialect(locale: string): Dialect {
	const lower = locale.toLowerCase();

	// Explicit matches
	if (lower.includes('en-gb') || lower.includes('en_gb')) return Dialect.British;
	if (lower.includes('en-au') || lower.includes('en_au')) return Dialect.Australian;
	if (lower.includes('en-ca') || lower.includes('en_ca')) return Dialect.Canadian;
	if (lower.includes('en-in') || lower.includes('en_in')) return Dialect.Indian;

	// Fallback for English variants
	if (lower.startsWith('en')) {
		// New Zealand → Australian (closest match)
		if (lower.includes('en-nz') || lower.includes('en_nz')) return Dialect.Australian;
		// Other English variants → British as fallback
		if (lower.match(/^en[-_]/)) return Dialect.British;
		// Plain 'en' → American (default)
		return Dialect.American;
	}

	// Non-English languages → American (fallback)
	return Dialect.American;
}

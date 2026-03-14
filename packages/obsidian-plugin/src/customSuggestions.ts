export type CustomReplacements = Record<string, string[]>;

export function stringToCustomReplacements(value: string): CustomReplacements {
	const replacements: CustomReplacements = {};

	for (const rawLine of value.split('\n')) {
		const line = rawLine.trim();
		if (line.length === 0) {
			continue;
		}

		const separator = line.includes('=>') ? '=>' : '->';
		if (!line.includes(separator)) {
			continue;
		}

		const [rawTypo, rawSuggestions] = line.split(separator, 2);
		const typo = rawTypo.trim();
		const suggestions = rawSuggestions
			.split(',')
			.map((suggestion) => suggestion.trim())
			.filter((suggestion) => suggestion.length > 0);

		if (typo.length === 0 || suggestions.length === 0) {
			continue;
		}

		replacements[typo] = suggestions;
	}

	return replacements;
}

export function customReplacementsToString(customReplacements?: CustomReplacements): string {
	if (!customReplacements) {
		return '';
	}

	return Object.entries(customReplacements)
		.map(([typo, suggestions]) => `${typo} -> ${suggestions.join(', ')}`)
		.join('\n');
}

export function cloneCustomReplacements(
	customReplacements?: CustomReplacements,
): CustomReplacements | undefined {
	if (!customReplacements) {
		return undefined;
	}

	return Object.fromEntries(
		Object.entries(customReplacements).map(([typo, suggestions]) => [typo, [...suggestions]]),
	);
}

export function normalizeCustomReplacements(
	customReplacements?: CustomReplacements,
): CustomReplacements {
	const normalized: CustomReplacements = {};

	for (const [rawTypo, rawSuggestions] of Object.entries(customReplacements ?? {})) {
		const typo = rawTypo.trim().toLowerCase();
		const deduped: string[] = [];
		const seen = new Set<string>();

		for (const rawSuggestion of rawSuggestions) {
			const suggestion = rawSuggestion.trim();
			const key = suggestion.toLowerCase();
			if (suggestion.length === 0 || seen.has(key)) {
				continue;
			}

			seen.add(key);
			deduped.push(suggestion);
		}

		if (typo.length === 0 || deduped.length === 0) {
			continue;
		}

		normalized[typo] = deduped;
	}

	return normalized;
}

export function getCustomSuggestions(
	problemText: string,
	customReplacements: CustomReplacements,
): string[] {
	const normalizedProblemText = problemText.trim().toLowerCase();
	const suggestions = customReplacements[normalizedProblemText] ?? [];

	const adapted: string[] = [];
	const seen = new Set<string>();

	for (const suggestion of suggestions) {
		const replacement = matchSimpleCase(problemText, suggestion);
		const key = replacement.toLowerCase();
		if (seen.has(key)) {
			continue;
		}

		seen.add(key);
		adapted.push(replacement);
	}

	return adapted;
}

function matchSimpleCase(problemText: string, replacement: string): string {
	const letters = [...problemText].filter((char) => /[A-Za-z]/.test(char));
	if (letters.length === 0) {
		return replacement;
	}

	if (letters.every((char) => char === char.toUpperCase())) {
		return replacement.toUpperCase();
	}

	const [first = ''] = letters;
	const rest = letters.slice(1);
	if (first === first.toUpperCase() && rest.every((char) => char === char.toLowerCase())) {
		return `${replacement.charAt(0).toUpperCase()}${replacement.slice(1)}`;
	}

	return replacement;
}

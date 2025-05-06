import { SuggestionKind } from 'harper.js';

/**
 * Produce the UI text shown inside suggestion buttons.
 * @param kind
 * @param problemText
 * @param replacementText
 */
export function suggestionText(
	kind: SuggestionKind,
	problemText: string,
	replacementText: string,
): string {
	if (kind === SuggestionKind.Remove) {
		return `Remove “${problemText}”`;
	} else if (kind === SuggestionKind.Replace) {
		return `Replace with “${replacementText}”`;
	}
	return `Insert “${replacementText}”`;
}

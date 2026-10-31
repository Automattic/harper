import { SuggestionKind } from 'harper.js';
import type { UnpackedLint } from '../types.js';

export const exampleLint: UnpackedLint = {
	span: { start: 4, end: 7 },
	message_html: 'Did you mean <code>the</code>?',
	problem_text: 'teh',
	lint_kind: 'Spelling',
	lint_kind_pretty: 'Spelling',
	suggestions: [{ kind: SuggestionKind.Replace, replacement_text: 'the' }],
	context_hash: 'example-spelling',
	source: 'Try teh editor.',
};

export const exampleSnippet = {
	prefix: 'Try ',
	problem: 'teh',
	suffix: ' editor.',
	prefixEllipsis: false,
	suffixEllipsis: false,
};

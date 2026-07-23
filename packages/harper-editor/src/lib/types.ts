export type { Linter as EditorLinter, LintKind } from 'harper.js';

export type {
	Box,
	IgnorableLintBox,
	LintBox,
	UnpackedLint,
	UnpackedSpan,
	UnpackedSuggestion,
} from 'lint-framework';

export type SourceTextNode = {
	textContent: string | null;
};

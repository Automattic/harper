import type { Linter } from 'harper.js';

export type EditorLinter = Linter;

export type {
	Box,
	IgnorableLintBox,
	LintBox,
	LintKind,
	UnpackedLint,
	UnpackedSpan,
	UnpackedSuggestion,
} from 'lint-framework';

export type SourceTextNode = {
	textContent: string | null;
};

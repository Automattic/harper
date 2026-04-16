import type { SuggestionKind } from 'harper.js';

export type LintKind =
	| 'Agreement'
	| 'BoundaryError'
	| 'Capitalization'
	| 'Eggcorn'
	| 'Enhancement'
	| 'Formatting'
	| 'Grammar'
	| 'Malapropism'
	| 'Miscellaneous'
	| 'Nonstandard'
	| 'Punctuation'
	| 'Readability'
	| 'Redundancy'
	| 'Regionalism'
	| 'Repetition'
	| 'Spelling'
	| 'Style'
	| 'Typo'
	| 'Usage'
	| 'WordChoice';

export type UnpackedSpan = {
	start: number;
	end: number;
};

export type UnpackedSuggestion = {
	kind: SuggestionKind;
	replacement_text: string;
};

export type UnpackedLint = {
	span: UnpackedSpan;
	message_html: string;
	problem_text: string;
	lint_kind: LintKind;
	lint_kind_pretty: string;
	suggestions: UnpackedSuggestion[];
	context_hash: string;
	source: string;
};

export type SourceTextNode = {
	textContent: string | null;
};

export type Box = {
	x: number;
	y: number;
	width: number;
	height: number;
};

export type LintBox = Box & {
	lint: UnpackedLint;
	source: SourceTextNode;
	range?: Range;
	applySuggestion: (sug: UnpackedSuggestion) => void;
};

export type IgnorableLintBox = LintBox & {
	rule: string;
	ignoreLint?: () => Promise<void>;
};

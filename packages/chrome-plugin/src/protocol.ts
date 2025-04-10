import type { UnpackedLint, UnpackedSuggestion } from './unpackLint';

export type Request = LintRequest | ApplySuggestionRequest;

export type LintRequest = {
	kind: 'lint';
	text: string;
};

export type ApplySuggestionRequest = {
	kind: 'apply';
	text: string;
	suggestion: UnpackedSuggestion;
};

export type Response = LintResponse;

export type LintResponse = {
	kind: 'lints';
	lints: UnpackedLint[];
};

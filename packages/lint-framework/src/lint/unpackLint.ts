import { type Lint, type Linter, SuggestionKind } from 'harper.js';
import type { LintKind } from './lintKindColor';

export type UnpackedSpan = {
	start: number;
	end: number;
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

export type UnpackedLintGroups = Record<string, UnpackedLint[]>;

export type UnpackedSuggestion = {
	kind: SuggestionKind;
	/// An empty string if replacement text is not applicable.
	replacement_text: string;
};

export default async function unpackLint(
	text: string,
	lint: Lint,
	linter: Linter,
): Promise<UnpackedLint> {
	const span = lint.span();

	return {
		span: { start: span.start, end: span.end },
		message_html: lint.message_html(),
		problem_text: lint.get_problem_text(),
		lint_kind: lint.lint_kind() as LintKind,
		lint_kind_pretty: lint.lint_kind_pretty(),
		suggestions: lint.suggestions().map((sug) => {
			return { kind: sug.kind(), replacement_text: sug.get_replacement_text() };
		}),
		context_hash: (await linter.contextHash(text, lint)).toString(),
		source: text,
	};
}

export function applySuggestion(text: string, span: UnpackedSpan, sug: UnpackedSuggestion): string {
	switch (sug.kind) {
		case SuggestionKind.Remove:
			return text.slice(0, span.start) + text.slice(span.end);
		case SuggestionKind.Replace:
			return text.slice(0, span.start) + sug.replacement_text + text.slice(span.end);
		case SuggestionKind.InsertAfter:
			return text.slice(0, span.end) + sug.replacement_text + text.slice(span.end);
	}
}

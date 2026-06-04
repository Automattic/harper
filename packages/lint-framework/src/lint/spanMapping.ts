import type { UnpackedLint, UnpackedSpan } from './unpackLint';

export type SingleTextEdit = {
	oldStart: number;
	oldEnd: number;
	newStart: number;
	newEnd: number;
	delta: number;
};

/**
 * Compute the smallest single changed range between two text snapshots.
 *
 * This is intentionally simple: during typing/debounce we only need to handle the
 * common single-edit case. Complex edits collapse into one wider edit, which makes
 * overlapping lints fail closed and disappear until the next lint pass.
 */
export function computeSingleTextEdit(previous: string, current: string): SingleTextEdit | null {
	if (previous === current) {
		return null;
	}

	let prefixLength = 0;
	const shortestLength = Math.min(previous.length, current.length);
	while (prefixLength < shortestLength && previous[prefixLength] === current[prefixLength]) {
		prefixLength++;
	}

	let suffixLength = 0;
	while (
		suffixLength < previous.length - prefixLength &&
		suffixLength < current.length - prefixLength &&
		previous[previous.length - 1 - suffixLength] === current[current.length - 1 - suffixLength]
	) {
		suffixLength++;
	}

	const oldStart = prefixLength;
	const oldEnd = previous.length - suffixLength;
	const newStart = prefixLength;
	const newEnd = current.length - suffixLength;

	return {
		oldStart,
		oldEnd,
		newStart,
		newEnd,
		delta: newEnd - newStart - (oldEnd - oldStart),
	};
}

/**
 * Remap a lint span across a single text edit.
 *
 * Spans before the edit are unchanged. Spans after the edit are shifted by the
 * edit delta. Spans touched by the edit are rejected because their lint contents,
 * suggestions, and context may no longer be valid.
 */
export function remapSpanForTextEdit(
	span: UnpackedSpan,
	edit: SingleTextEdit | null,
): UnpackedSpan | null {
	if (edit == null) {
		return { ...span };
	}

	if (span.end <= edit.oldStart) {
		return { ...span };
	}

	if (span.start >= edit.oldEnd) {
		return {
			start: span.start + edit.delta,
			end: span.end + edit.delta,
		};
	}

	return null;
}

/**
 * Return a lint whose span is valid for `currentSource`, or `null` if it should
 * be hidden until the next lint pass.
 */
export function remapLintToCurrentSource(
	lint: UnpackedLint,
	currentSource: string,
): UnpackedLint | null {
	if (lint.source === currentSource) {
		return lint;
	}

	const span = remapSpanForTextEdit(lint.span, computeSingleTextEdit(lint.source, currentSource));

	if (span == null) {
		return null;
	}

	if (currentSource.slice(span.start, span.end) !== lint.problem_text) {
		return null;
	}

	return {
		...lint,
		span,
		source: currentSource,
	};
}

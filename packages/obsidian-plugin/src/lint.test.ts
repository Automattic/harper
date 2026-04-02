import type { EditorView } from '@codemirror/view';
import { expect, test } from 'vitest';
import { canApplySuggestionFromVisibleTooltip } from './lint';

test('canApplySuggestionFromVisibleTooltip handles null tooltip entries', () => {
	const mockView = {
		state: {
			field: () => ({
				commandTooltip: null,
				diagnostics: {
					between: () => {},
				},
			}),
			facet: () => [null],
		},
	} as unknown as EditorView;

	expect(() => canApplySuggestionFromVisibleTooltip(mockView, 1)).not.toThrow();
	expect(canApplySuggestionFromVisibleTooltip(mockView, 1)).toBe(false);
});

import { expect, test } from 'vitest';
import { lintKindClass, lintKindColor } from './lintKindColor';

test('lintKindColor matches Harper web colors for common kinds', () => {
	expect(lintKindColor('Spelling')).toBe('#EE4266');
	expect(lintKindColor('Style')).toBe('#FFD23F');
	expect(lintKindColor('Grammar')).toBe('#9B59B6');
});

test('lint kind helpers produce safe fallback values', () => {
	expect(lintKindColor('DoesNotExist')).toBe('#d11');
	expect(lintKindClass('Style')).toBe('harper-lintRange-Style');
});

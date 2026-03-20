const FALLBACK_LINT_COLOR = '#d11';

export const LINT_KIND_COLORS = {
	Agreement: '#228B22',
	BoundaryError: '#8B4513',
	Capitalization: '#540D6E',
	Eggcorn: '#FF8C00',
	Enhancement: '#0EAD69',
	Formatting: '#7D3C98',
	Grammar: '#9B59B6',
	Malapropism: '#C71585',
	Miscellaneous: '#3BCEAC',
	Nonstandard: '#008B8B',
	Punctuation: '#D4850F',
	Readability: '#2E8B57',
	Redundancy: '#4682B4',
	Regionalism: '#C061CB',
	Repetition: '#00A67C',
	Spelling: '#EE4266',
	Style: '#FFD23F',
	Typo: '#FF6B35',
	Usage: '#1E90FF',
	WordChoice: '#228B22',
} as const;

export function lintKindColor(lintKindKey: string): string {
	return LINT_KIND_COLORS[lintKindKey as keyof typeof LINT_KIND_COLORS] ?? FALLBACK_LINT_COLOR;
}

export function lintKindClass(lintKindKey: string): string {
	return `harper-lintRange-${lintKindKey}`;
}

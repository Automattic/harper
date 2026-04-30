import type { IgnorableLintBox, LintBox, UnpackedSuggestion } from './types.js';

export type EditorFontFamily = 'sans' | 'serif' | 'mono';

export type EditorFontSize = 'default' | number;

export type LintDisplayCategory =
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

export type CategoryCounts = Record<LintDisplayCategory, number>;

export const FONT_OPTIONS: {
	value: EditorFontFamily;
	label: string;
	sample: string;
	stack: string;
}[] = [
	{
		value: 'sans',
		label: 'Sans',
		sample: 'Aa',
		stack:
			"'Inter', -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif",
	},
	{
		value: 'serif',
		label: 'Serif',
		sample: 'Aa',
		stack: "'Iowan Old Style', 'New York', Georgia, serif",
	},
	{
		value: 'mono',
		label: 'Mono',
		sample: 'Aa',
		stack: "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace",
	},
];

export const FONT_SIZES = [12, 13, 14, 15, 16, 17, 18, 20, 22];
export const DEFAULT_FONT_SIZE = 'default';

export const LINT_CATEGORY_ORDER: LintDisplayCategory[] = [
	'Spelling',
	'Typo',
	'Capitalization',
	'Grammar',
	'Agreement',
	'BoundaryError',
	'Punctuation',
	'Usage',
	'WordChoice',
	'Style',
	'Readability',
	'Enhancement',
	'Redundancy',
	'Repetition',
	'Formatting',
	'Regionalism',
	'Nonstandard',
	'Eggcorn',
	'Malapropism',
	'Miscellaneous',
];

export const LINT_CATEGORIES: Record<
	LintDisplayCategory,
	{
		label: string;
		dotClass: string;
		haloClass: string;
		textClass: string;
		softClass: string;
		activeClass: string;
	}
> = {
	Agreement: {
		label: 'Agreement',
		dotClass: 'bg-emerald-500',
		haloClass: 'bg-emerald-100',
		textClass: 'text-emerald-700',
		softClass: 'bg-emerald-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-emerald-500/10',
	},
	BoundaryError: {
		label: 'Boundary',
		dotClass: 'bg-indigo-500',
		haloClass: 'bg-indigo-100',
		textClass: 'text-indigo-700',
		softClass: 'bg-indigo-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-indigo-500/10',
	},
	Capitalization: {
		label: 'Capitalization',
		dotClass: 'bg-rose-500',
		haloClass: 'bg-rose-100',
		textClass: 'text-rose-700',
		softClass: 'bg-rose-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-rose-500/10',
	},
	Eggcorn: {
		label: 'Eggcorn',
		dotClass: 'bg-violet-500',
		haloClass: 'bg-violet-100',
		textClass: 'text-violet-700',
		softClass: 'bg-violet-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-violet-500/10',
	},
	Enhancement: {
		label: 'Enhancement',
		dotClass: 'bg-amber-600',
		haloClass: 'bg-amber-100',
		textClass: 'text-amber-700',
		softClass: 'bg-amber-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-amber-600/10',
	},
	Formatting: {
		label: 'Formatting',
		dotClass: 'bg-slate-500',
		haloClass: 'bg-slate-100',
		textClass: 'text-slate-700',
		softClass: 'bg-slate-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-slate-500/10',
	},
	Grammar: {
		label: 'Grammar',
		dotClass: 'bg-emerald-600',
		haloClass: 'bg-emerald-100',
		textClass: 'text-emerald-700',
		softClass: 'bg-emerald-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-emerald-600/10',
	},
	Malapropism: {
		label: 'Malapropism',
		dotClass: 'bg-fuchsia-500',
		haloClass: 'bg-fuchsia-100',
		textClass: 'text-fuchsia-700',
		softClass: 'bg-fuchsia-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-fuchsia-500/10',
	},
	Miscellaneous: {
		label: 'Miscellaneous',
		dotClass: 'bg-sky-500',
		haloClass: 'bg-sky-100',
		textClass: 'text-sky-700',
		softClass: 'bg-sky-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-sky-500/10',
	},
	Nonstandard: {
		label: 'Nonstandard',
		dotClass: 'bg-stone-500',
		haloClass: 'bg-stone-100',
		textClass: 'text-stone-700',
		softClass: 'bg-stone-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-stone-500/10',
	},
	Punctuation: {
		label: 'Punctuation',
		dotClass: 'bg-cyan-600',
		haloClass: 'bg-cyan-100',
		textClass: 'text-cyan-700',
		softClass: 'bg-cyan-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-cyan-600/10',
	},
	Readability: {
		label: 'Readability',
		dotClass: 'bg-violet-600',
		haloClass: 'bg-violet-100',
		textClass: 'text-violet-700',
		softClass: 'bg-violet-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-violet-600/10',
	},
	Redundancy: {
		label: 'Redundancy',
		dotClass: 'bg-amber-600',
		haloClass: 'bg-amber-100',
		textClass: 'text-amber-700',
		softClass: 'bg-amber-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-amber-600/10',
	},
	Regionalism: {
		label: 'Regionalism',
		dotClass: 'bg-teal-700',
		haloClass: 'bg-teal-100',
		textClass: 'text-teal-700',
		softClass: 'bg-teal-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-teal-700/10',
	},
	Repetition: {
		label: 'Repetition',
		dotClass: 'bg-amber-700',
		haloClass: 'bg-amber-100',
		textClass: 'text-amber-700',
		softClass: 'bg-amber-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-amber-700/10',
	},
	Spelling: {
		label: 'Spelling',
		dotClass: 'bg-rose-500',
		haloClass: 'bg-rose-100',
		textClass: 'text-rose-700',
		softClass: 'bg-rose-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-rose-500/10',
	},
	Style: {
		label: 'Style',
		dotClass: 'bg-blue-500',
		haloClass: 'bg-blue-100',
		textClass: 'text-blue-700',
		softClass: 'bg-blue-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-blue-500/10',
	},
	Typo: {
		label: 'Typo',
		dotClass: 'bg-rose-500',
		haloClass: 'bg-rose-100',
		textClass: 'text-rose-700',
		softClass: 'bg-rose-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-rose-500/10',
	},
	Usage: {
		label: 'Usage',
		dotClass: 'bg-green-600',
		haloClass: 'bg-green-100',
		textClass: 'text-green-700',
		softClass: 'bg-green-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-green-600/10',
	},
	WordChoice: {
		label: 'Word Choice',
		dotClass: 'bg-blue-600',
		haloClass: 'bg-blue-100',
		textClass: 'text-blue-700',
		softClass: 'bg-blue-50',
		activeClass: 'border-[rgba(28,26,22,0.14)] shadow-blue-600/10',
	},
};

export const LINT_CATEGORY_ENTRIES = LINT_CATEGORY_ORDER.map(
	(key) => [key, LINT_CATEGORIES[key]] as const,
);

export function createEmptyCategoryCounts(): CategoryCounts {
	return Object.fromEntries(LINT_CATEGORY_ORDER.map((key) => [key, 0])) as CategoryCounts;
}

export function normalizeFontFamily(value: string): EditorFontFamily {
	if (value === 'serif' || value === 'mono') {
		return value;
	}

	return 'sans';
}

export function fontStackFor(value: EditorFontFamily): string {
	return FONT_OPTIONS.find((option) => option.value === value)?.stack ?? FONT_OPTIONS[0].stack;
}

export function normalizeFontSize(value: EditorFontSize | string): EditorFontSize {
	if (value === DEFAULT_FONT_SIZE || value == null || value === '') {
		return DEFAULT_FONT_SIZE;
	}

	const numericValue = typeof value === 'number' ? value : Number(value);
	if (!Number.isFinite(numericValue)) {
		return DEFAULT_FONT_SIZE;
	}

	const rounded = Math.round(numericValue);
	return Math.min(28, Math.max(11, rounded));
}

export function displayCategoryFor(lintKind: string): LintDisplayCategory {
	if (lintKind in LINT_CATEGORIES) {
		return lintKind as LintDisplayCategory;
	}

	return 'Miscellaneous';
}

export function lintBoxId(lintBox: IgnorableLintBox | LintBox): string {
	const { lint } = lintBox;
	const rule = 'rule' in lintBox ? lintBox.rule : '';
	return [rule, lint.context_hash, lint.span.start, lint.span.end, lint.problem_text].join(':');
}

export function suggestionText(suggestion: UnpackedSuggestion): string {
	return suggestion.replacement_text !== '' ? suggestion.replacement_text : String(suggestion.kind);
}

export function wordCount(text: string): number {
	return text.trim().match(/\S+/g)?.length ?? 0;
}

export function characterCount(text: string): number {
	return text.length;
}

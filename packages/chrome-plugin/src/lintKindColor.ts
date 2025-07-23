// First, define the color map as a constant
const LINT_KIND_COLORS = {
	Agreement: '#228B22', // Forest green
	BoundaryError: '#8B4513', // Saddle brown
	Capitalization: '#540D6E', // Deep purple
	Eggcorn: '#FF8C00', // Dark orange
	Enhancement: '#0EAD69', // Green
	Formatting: '#7D3C98', // Amethyst purple
	Grammar: '#9B59B6', // Medium purple
	Malapropism: '#C71585', // Medium violet red
	Miscellaneous: '#3BCEAC', // Turquoise
	Nonstandard: '#008B8B', // Dark cyan
	Punctuation: '#D4850F', // Dark orange
	Readability: '#2E8B57', // Sea green
	Redundancy: '#4682B4', // Steel blue
	Regionalism: '#C061CB', // Vibrant purple
	Repetition: '#00A67C', // Green-cyan
	Spelling: '#EE4266', // Pink-red
	Style: '#FFD23F', // Yellow
	Typo: '#FF6B35', // Vibrant orange-red
	Usage: '#1E90FF', // Dodger blue
	WordChoice: '#228B22', // Forest green
} as const;

// Export the type for the lint kind keys
export type LintKind = keyof typeof LINT_KIND_COLORS;

// Export the array of all lint kind names
export const LINT_KINDS = Object.keys(LINT_KIND_COLORS) as LintKind[];

// The main function that uses the map
export default function lintKindColor(lintKindKey: string): string {
	const color = LINT_KIND_COLORS[lintKindKey as LintKind];
	if (!color) {
		throw new Error(`Unexpected lint kind: ${lintKindKey}`);
	}
	return color;
}

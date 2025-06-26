export default function lintKindColor(lintKindKey: string): string {
	switch (lintKindKey) {
		case 'Capitalization':
			return '#540D6E'; // Deep purple
		case 'Enhancement':
			return '#0EAD69'; // Green
		case 'Formatting':
			return '#7D3C98'; // Amethyst purple
		case 'Miscellaneous':
			return '#3BCEAC'; // Turquoise
		case 'Punctuation':
			return '#D4850F'; // Dark orange
		case 'Readability':
			return '#2E8B57'; // Sea green
		case 'Redundancy':
			return '#1A5FB4'; // Deep blue
		case 'Regionalism':
			return '#C061CB'; // Vibrant purple
		case 'Repetition':
			return '#00A67C'; // Green-cyan
		case 'Spelling':
			return '#EE4266'; // Pink-red
		case 'Style':
			return '#FFD23F'; // Yellow
		case 'Typo':
			return '#FF6B35'; // Vibrant orange-red
		case 'WordChoice':
			return '#228B22'; // Forest green
		default:
			throw new Error(`Unexpected lint kind: ${lintKindKey}`);
	}
}

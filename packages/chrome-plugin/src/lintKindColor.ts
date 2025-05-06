export default function lintKindColor(lintKindKey: string): string {
	switch (lintKindKey) {
		case 'Spelling':
			return '#EE4266';
		case 'Capitalization':
			return '#540D6E';
		case 'Style':
			return '#FFD23F';
		case 'Formatting':
			return '#540D6E';
		case 'Repetition':
			return '#3BCEAC';
		case 'Enhancement':
			return '#0EAD69';
		case 'Readability':
			return '#0EAD69';
		case 'WordChoice':
			return '#0EAD69';
		case 'Miscellaneous':
			return '#3BCEAC';
		case 'Punctuation':
			return '#D4850F';
		default:
			throw new Error(`Unexpected lint kind: ${lintKindKey}`);
	}
}

/** Converts the content of a text area to individual lines. */
export function stringToLines(s: string): string[] {
	return s
		.split('\n')
		.map((s) => s.trim())
		.filter((v) => v.length > 0);
}

/** Converts the content of a text area to viable dictionary values. */
export function linesToString(values: string[]): string {
	return values.map((v) => v.trim()).join('\n');
}

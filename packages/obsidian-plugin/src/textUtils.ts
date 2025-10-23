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

/**
 * Convert Harper's character index into a UTF-16 code unit index understood by Obsidian.
 * `sourceChars` should contain strings produced from Rust `char`s (i.e. Unicode scalar values).
 */
export function charIndexToCodeUnit(index: number, sourceChars: string[]): number {
	if (index < 0 || index > sourceChars.length) {
		throw new RangeError(
			`Character index ${index} is out of bounds for source length ${sourceChars.length}`,
		);
	}

	let codeUnitIndex = 0;

	for (let i = 0; i < index; i += 1) {
		const current = sourceChars[i];

		if (typeof current !== 'string') {
			throw new TypeError(`Expected string at char index ${i}, received ${typeof current}`);
		}

		codeUnitIndex += current.length;
	}

	return codeUnitIndex;
}

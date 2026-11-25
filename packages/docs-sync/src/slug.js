/**
 * Heading ids keep their capitals: "Updating the Extension" is `Updating-the-Extension`.
 * Shipped clients and the docs themselves link to ids in this form, so it must not change.
 */
export function headingId(text) {
	return text
		.replace(/[^\p{L}\p{N}\s_-]/gu, '')
		.trim()
		.replace(/\s+/g, '-');
}

/** Two headings with the same text on one page get `Name` and `Name-1`. */
export function uniqueId(id, used) {
	let candidate = id || 'section';

	for (let n = 1; used.has(candidate); n++) {
		candidate = `${id || 'section'}-${n}`;
	}

	used.add(candidate);
	return candidate;
}

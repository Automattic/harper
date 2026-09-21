/** Given a slug of the form `"lastweek" | "last30days" | "lastday" | "all"`, compute a representative duration. */
export function computeDurationFromSlug(slug: string): number | null {
	switch (slug) {
		case 'last30days':
			return 30 * 24 * 60 * 60 * 1000;
		case 'lastday':
			return 24 * 60 * 60 * 1000;
		case 'lastweek':
			return 24 * 60 * 60 * 1000 * 7;
		case 'all':
			return Date.now();
	}
}

/** Count the occurances of unique strings in an array. */
export function countOccurances(arr: string[]): Record<string, number> {
	const counts: Record<string, number> = {};

	for (const item of arr) {
		if (counts[item] === undefined) {
			counts[item] = 1;
		} else {
			counts[item] += 1;
		}
	}

	return counts;
}

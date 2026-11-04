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
		default:
			return null;
	}
}

/** Count the occurrences of unique strings in an array. */
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

/**
 * Count reports by UTC day in date order, including zero-count days.
 * Reports must already be filtered to [start, end); both bounds must be UTC midnight.
 * A null start uses the first report's day, or returns no points if none exist.
 */
export function countReportsByDay(
	reports: readonly { timestamp: Date }[],
	start: Date | null,
	end: Date,
): { date: Date; value: number }[] {
	const dayLength = 24 * 60 * 60 * 1000;
	const counts = new Map<number, number>();
	let firstDay = start?.getTime() ?? Number.POSITIVE_INFINITY;

	for (const report of reports) {
		const day = Math.floor(report.timestamp.getTime() / dayLength) * dayLength;
		counts.set(day, (counts.get(day) ?? 0) + 1);
		firstDay = Math.min(firstDay, day);
	}

	const points: { date: Date; value: number }[] = [];
	for (let day = firstDay; day < end.getTime(); day += dayLength) {
		points.push({ date: new Date(day), value: counts.get(day) ?? 0 });
	}
	return points;
}

import { redirect } from '@sveltejs/kit';
import { computeDurationFromSlug } from '$lib/adminUtils';
import ProblematicLints from '$lib/db/models/ProblematicLints';
import type { PageServerLoad } from './$types';

/**
 * Count reports by UTC calendar day, including zero-count days, in date order.
 * Start is inclusive and end is exclusive; both must be UTC midnight.
 * A null start uses the first report's day, or returns no points if none exist.
 */
function buildDailyReportCounts(
	timestamps: readonly Date[],
	start: Date | null,
	end: Date,
): { date: Date; value: number }[] {
	const dayLength = 24 * 60 * 60 * 1000;
	const startTime = start?.getTime() ?? 0;
	const endTime = end.getTime();
	const counts = new Map<number, number>();
	let firstTime = start?.getTime() ?? Number.POSITIVE_INFINITY;

	for (const timestamp of timestamps) {
		const time = timestamp.getTime();
		if (!Number.isFinite(time) || time < startTime || time >= endTime) continue;

		const day = Math.floor(time / dayLength) * dayLength;
		counts.set(day, (counts.get(day) ?? 0) + 1);
		if (start === null) firstTime = Math.min(firstTime, time);
	}

	const points: { date: Date; value: number }[] = [];
	for (let day = Math.floor(firstTime / dayLength) * dayLength; day < endTime; day += dayLength) {
		points.push({ date: new Date(day), value: counts.get(day) ?? 0 });
	}
	return points;
}

/**
 * Load one lint ID's counts for complete UTC days, ending with yesterday.
 * All-time data starts at its first report's day and is empty if none exist.
 */
export const load: PageServerLoad = async ({ params }) => {
	const { slug, lintId } = params;
	const duration = computeDurationFromSlug(slug);

	if (duration === null) {
		redirect(302, `/admin/problematic-lints/all/${encodeURIComponent(lintId)}`);
	}

	const end = new Date();
	end.setUTCHours(0, 0, 0, 0);
	const start = slug === 'all' ? null : new Date(end.getTime() - duration);
	// getAllBetween includes its upper bound; exclude the start of today.
	const reports = await ProblematicLints.getAllBetween(
		start ?? new Date(0),
		new Date(end.getTime() - 1),
	);
	const timestamps = reports
		.filter((report) => (report.rule_id ?? 'OTHER') === lintId)
		.map((report) => report.timestamp);

	return {
		lintId,
		points: buildDailyReportCounts(timestamps, start, end),
	};
};

import { redirect } from '@sveltejs/kit';
import { computeDurationFromSlug } from '$lib/adminUtils';
import ProblematicLints from '$lib/db/models/ProblematicLints';
import type { PageServerLoad } from './$types';

/**
 * Load one lint ID's UTC daily counts, including today's partial total.
 * All-time data starts at its first report's day and is empty if none exist.
 */
export const load: PageServerLoad = async ({ params }) => {
	const { slug, lintId } = params;
	const duration = computeDurationFromSlug(slug);

	if (duration === null) {
		redirect(302, `/admin/problematic-lints/all/${encodeURIComponent(lintId)}`);
	}

	const end = new Date();
	end.setUTCHours(24, 0, 0, 0);
	const start = slug === 'all' ? null : new Date(end.getTime() - duration);

	return {
		lintId,
		points: await ProblematicLints.getDailyReportCounts(lintId, start, end),
	};
};

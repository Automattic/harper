import { redirect } from '@sveltejs/kit';
import { computeDurationFromSlug } from '$lib/adminUtils';
import UninstallFeedback from '$lib/db/models/UninstallFeedback';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	const { slug, reason } = params;
	const duration = computeDurationFromSlug(slug);

	if (duration === null) {
		redirect(302, `/admin/ext-uninstall-reasons/all/${encodeURIComponent(reason)}`);
	}

	const end = new Date();
	end.setUTCHours(0, 0, 0, 0);
	const start = slug === 'all' ? null : new Date(end.getTime() - duration);

	return {
		reason,
		points: await UninstallFeedback.getDailyReportCounts(reason, start, end),
	};
};

import { redirect } from '@sveltejs/kit';
import { computeDurationFromSlug } from '$lib/adminUtils';
import DomainReviews from '$lib/db/models/DomainReviews';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	const { slug, domain } = params;
	const duration = computeDurationFromSlug(slug);

	if (duration === null) {
		redirect(302, `/admin/ext-site-problems/all/${encodeURIComponent(domain)}`);
	}

	const end = new Date();
	end.setUTCHours(0, 0, 0, 0);
	const start = slug === 'all' ? null : new Date(end.getTime() - duration);

	return {
		domain,
		points: await DomainReviews.getDailyReportCounts(domain, start, end),
	};
};

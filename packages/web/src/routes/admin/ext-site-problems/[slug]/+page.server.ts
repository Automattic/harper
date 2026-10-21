import { redirect } from '@sveltejs/kit';
import { computeDurationFromSlug, countOccurances } from '$lib/adminUtils';
import DomainReviews from '$lib/db/models/DomainReviews';

export const load = async ({ params }) => {
	const slug = params.slug;

	const duration = computeDurationFromSlug(slug);

	if (duration == null) {
		redirect(302, '/admin/ext-site-problems/all');
	}

	const date = Date.now() - duration;

	const domainReviews = await DomainReviews.getAllSince(new Date(date));
	const prevDomainReviews = await DomainReviews.getAllBetween(
		new Date(date - duration),
		new Date(date),
	);

	const counts: Record<string, number> = countOccurances(
		domainReviews.map((i) => i.domain ?? 'OTHER'),
	);
	const prevCounts: Record<string, number> = countOccurances(
		prevDomainReviews.map((i) => i.domain ?? 'OTHER'),
	);

	return {
		counts,
		prevCounts,
	};
};

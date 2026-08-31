import { redirect } from '@sveltejs/kit';
import DomainReviews from '$lib/db/models/DomainReviews';

export const load = async ({ params }) => {
	const slug = params.slug;

	let date = null;

	switch (slug) {
		case 'last30days':
			date = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
			break;
		case 'lastday':
			date = new Date(Date.now() - 24 * 60 * 60 * 1000);
			break;
		case 'all':
			date = new Date(0);
			break;
	}

	if (date == null) {
		redirect(302, '/admin/ext-site-problems/all');
	}

	const domainReviews = await DomainReviews.getAllSince(date);

	const counts: Record<string, number> = {};

	for (const item of domainReviews) {
		const id = item.domain ?? 'OTHER';

		if (counts[id] === undefined) {
			counts[id] = 1;
		} else {
			counts[id] += 1;
		}
	}

	return {
		counts,
	};
};

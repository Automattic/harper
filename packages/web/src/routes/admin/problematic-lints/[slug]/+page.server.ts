import { redirect } from '@sveltejs/kit';
import { computeDurationFromSlug, countOccurances } from '$lib/adminUtils';
import ProblematicLints from '$lib/db/models/ProblematicLints';

export const load = async ({ params }) => {
	const slug = params.slug;

	const duration = computeDurationFromSlug(slug);

	if (duration == null) {
		redirect(302, '/admin/problematic-lints/all');
	}

	const date = Date.now() - duration;

	const problematicLints = await ProblematicLints.getAllSince(new Date(date));
	const prevProblematicLints = await ProblematicLints.getAllBetween(
		new Date(date - duration),
		new Date(date),
	);

	const counts: Record<string, number> = countOccurances(
		problematicLints.map((i) => i.rule_id ?? 'OTHER'),
	);
	const prevCounts: Record<string, number> = countOccurances(
		prevProblematicLints.map((i) => i.rule_id ?? 'OTHER'),
	);

	return {
		counts,
		prevCounts,
	};
};

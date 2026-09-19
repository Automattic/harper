import { redirect } from '@sveltejs/kit';
import UninstallFeedback from '$lib/db/models/UninstallFeedback';
import { countOccurances } from '$lib/adminUtils';

export const load = async ({ params }) => {
	const slug = params.slug;

  let duration = computeDurationFromSlug(slug);

	if (duration == null) {
		redirect(302, '/admin/ext-uninstall-reasons/all');
	}
  
  const date = Date.now() - duration;

	const uninstallFeedback = await UninstallFeedback.getAllSince(new Date(date));
	const prevUninstallFeedback = await UninstallFeedback.getAllBetween(new Date(date - duration), new Date(date));

	const counts: Record<string, number> = countOccurances(uninstallFeedback.map(i => i.feedback ?? "OTHER"));
	const prevCounts: Record<string, number> = countOccurances(prevUninstallFeedback.map(i => i.feedback ?? "OTHER"));

	return {
		counts,
    prevCounts
	};
};

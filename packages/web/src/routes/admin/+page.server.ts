import ProblematicLints from '$lib/db/models/ProblematicLints';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const problematicLints = await ProblematicLints.getAll();

	const counts: Record<string, number> = {};

	for (const item of problematicLints) {
		const id = item.rule_id ?? 'OTHER';

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

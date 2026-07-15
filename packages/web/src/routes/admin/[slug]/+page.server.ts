import ProblematicLints from '$lib/db/models/ProblematicLints';
import type { PageLoad } from './$types';

export const load = async ({ params }) => {
  const slug = params.slug;

  let date = null; 

  switch (slug){
    case 'last30days':
      date = new Date(
	      Date.now() - 30 * 24 * 60 * 60 * 1000,
      );
      break;
    case 'lastday':
      date = new Date(
	      Date.now() -  24 * 60 * 60 * 1000,
      );
      break;
    case 'all':
      date =  new Date(0);
      break;
  }

  if (date == null){
    return { counts: {}}
  }

	const problematicLints = await ProblematicLints.getAllSince(date);

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

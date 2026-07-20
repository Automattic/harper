import { error, type RequestEvent, redirect } from '@sveltejs/kit';
import ProblematicDomains from '$lib/db/models/ProblematicDomains';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.formData();

	const worksText = data.get('works');
	let works = null;

	switch (worksText) {
		case 'yes':
			works = true;
			break;
		case 'no':
			works = false;
			break;
	}

	if (works === null) {
		error(400, '`works` must be either yes or no.');
	}

	await ProblematicDomains.validateAndCreate({
		domain: data.get('domain'),
		works,
		feedback: data.get('feedback'),
	});

	throw redirect(303, '/');
};

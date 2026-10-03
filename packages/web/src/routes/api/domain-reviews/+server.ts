import { error, type RequestEvent, redirect } from '@sveltejs/kit';
import DomainReviews from '$lib/db/models/DomainReviews';
import { parseRequestBody } from '$lib/requestBody';

export const POST = async ({ request }: RequestEvent) => {
	const data = await parseRequestBody(request);

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
		throw error(400, '`works` must be either yes or no.');
	}

	await DomainReviews.validateAndCreate({
		domain: data.get('domain'),
		works,
		feedback: data.get('feedback'),
	});

	throw redirect(303, '/');
};

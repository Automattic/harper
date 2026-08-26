import { error, type RequestEvent, redirect } from '@sveltejs/kit';
import DomainReviews from '$lib/db/models/DomainReviews';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.json();

	const worksText = data.works;
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

	await DomainReviews.validateAndCreate({
		domain: data.domain,
		works,
		feedback: data.feedback,
	});

	throw redirect(303, '/');
};

import { error, type RequestEvent, redirect } from '@sveltejs/kit';
import DomainReviews from '$lib/db/models/DomainReviews';

export const POST = async ({ request }: RequestEvent) => {
	// The website's own <form> submits url-encoded (same-origin, unaffected by
	// SvelteKit's CSRF check); the browser extension sends JSON instead of
	// multipart/form-data specifically to sidestep that check cross-origin.
	const isJson = request.headers.get('content-type')?.startsWith('application/json');
	const data = isJson ? await request.json() : Object.fromEntries(await request.formData());

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

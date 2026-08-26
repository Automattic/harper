import { type RequestEvent, redirect } from '@sveltejs/kit';
import ProblematicLints from '$lib/db/models/ProblematicLints';

export const POST = async ({ request }: RequestEvent) => {
	// The website's own <form> submits url-encoded (same-origin, unaffected by
	// SvelteKit's CSRF check); the browser extension sends JSON instead of
	// multipart/form-data specifically to sidestep that check cross-origin.
	const isJson = request.headers.get('content-type')?.startsWith('application/json');
	const data = isJson ? await request.json() : Object.fromEntries(await request.formData());

	await ProblematicLints.validateAndCreate({
		is_false_positive: data.is_false_positive === 'true',
		example: data.example,
		rule_id: data.rule_id,
		feedback: data.feedback,
	});

	throw redirect(303, '/');
};


import { type RequestEvent, redirect } from '@sveltejs/kit';
import ProblematicDomains from '$lib/db/models/ProblematicDomains';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.formData();

	await ProblematicDomains.validateAndCreate({
		domain: data.get('domain'),
		feedback: data.get('feedback'),
	});

	throw redirect(303, '/');
};

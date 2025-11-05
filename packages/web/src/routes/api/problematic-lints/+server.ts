import { type RequestEvent, redirect } from '@sveltejs/kit';
import ProblematicLints from '$lib/db/models/ProblematicLints';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.formData();

	await ProblematicLints.validateAndCreate({
		is_false_positive: data.get('is_false_positive') === 'true',
		example: data.get('example'),
		rule_id: data.get('rule_id'),
		feedback: data.get('feedback'),
	});

	throw redirect(303, '/');
};

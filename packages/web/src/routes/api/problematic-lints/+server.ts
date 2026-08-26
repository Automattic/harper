import { type RequestEvent, redirect } from '@sveltejs/kit';
import ProblematicLints from '$lib/db/models/ProblematicLints';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.json();

	await ProblematicLints.validateAndCreate({
		is_false_positive: data.is_false_positive === 'true',
		example: data.example,
		rule_id: data.rule_id,
		feedback: data.feedback,
	});

	throw redirect(303, '/');
};

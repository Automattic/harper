import { type RequestEvent, redirect } from '@sveltejs/kit';
import UninstallFeedback from '$lib/db/models/UninstallFeedback';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.formData();

	await UninstallFeedback.validateAndCreate({
		feedback: data.get('feedback'),
	});
	throw redirect(303, '/');
};

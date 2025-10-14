import UninstallFeedback from '$lib/db/models/UninstallFeedback';
import { type RequestEvent, redirect } from '@sveltejs/kit';

export const POST = async ({ request }: RequestEvent) => {
	const data = await request.formData();

	await UninstallFeedback.validateAndCreate({
		feedback: data.get('feedback'),
	});
	throw redirect(303, '/');
};

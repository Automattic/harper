import UpdateCheckCounts from '$lib/db/models/UpdateCheckCounts';

export const load = async () => {
	const all = await UpdateCheckCounts.getAll();

	return {
		all,
	};
};

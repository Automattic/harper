import { redirect } from '@sveltejs/kit';
import { GithubClient } from '$lib/GitHubClient';
import type { RequestHandler } from './$types';

const latestReleaseUrl = 'https://github.com/Automattic/harper/releases/latest';
const desktopDmgPattern = /^(?:Harper|harper-desktop)_.*_universal\.dmg$/;

export const GET: RequestHandler = async () => {
	let downloadUrl: string | null = null;

	try {
		downloadUrl = await GithubClient.getLatestReleaseAssetUrlFromCache(
			'Automattic',
			'harper',
			desktopDmgPattern,
		);
	} catch (error) {
		console.error('Unable to resolve latest Harper Desktop download URL.', error);
	}

	throw redirect(302, downloadUrl ?? latestReleaseUrl);
};

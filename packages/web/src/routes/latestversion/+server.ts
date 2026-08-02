import { GithubClient } from '$lib/GitHubClient';

export async function GET() {
	const latestVersion = await GithubClient.getLatestReleaseFromCache('automattic', 'harper');

	if (latestVersion == null) {
		throw new Error('Unable to get latest version.');
	}

	console.log(`Received request for latest version. Responding with ${latestVersion}`);

	return new Response(latestVersion, {
		headers: {
			'Access-Control-Allow-Origin': '*',
			'Cache-Control': 'no-cache',
		},
	});
}

export async function OPTIONS() {
	return new Response(null, {
		headers: {
			'Access-Control-Allow-Origin': '*',
			'Access-Control-Allow-Methods': 'GET',
			'Access-Control-Allow-Headers': 'Harper-Version',
		},
	});
}

const { readFile, stat } = require('node:fs/promises');
const { basename } = require('node:path');
const { setTimeout: sleep } = require('node:timers/promises');

// Attach Windows artifacts to the tag's draft release. This script never creates
// a release: the macOS Buildkite step and the GitHub Actions workflows do that.
const REPOSITORY = 'Automattic/harper';

// The macOS step creates its draft earlier in the same build and is the only job
// that uploads a .dmg, so that draft is the one to use when GitHub Actions jobs
// have created others for the same tag.
function selectDraft(releases, tag) {
	const matches = releases.filter((release) => release.tag_name === tag);
	if (matches.some((release) => release.draft !== true)) {
		throw new Error(`Refusing to upload assets to published release ${tag}.`);
	}
	if (matches.length === 0) {
		throw new Error(
			`No draft release for ${tag} yet. Retry once the macOS or GitHub Actions jobs have created one.`,
		);
	}
	const candidates =
		matches.length === 1
			? matches
			: matches.filter((release) => release.assets.some((asset) => asset.name.endsWith('.dmg')));
	if (candidates.length !== 1) {
		const ids = matches.map((release) => release.id).join(', ');
		throw new Error(
			`Expected one draft for ${tag} holding the macOS .dmg; found ${matches.length} drafts (ids ${ids}). Reconcile them before retrying.`,
		);
	}
	return candidates[0];
}

async function uploadRelease({ repository, tag, assets, token, request = fetch, wait = sleep }) {
	if (!token?.trim()) throw new Error('GITHUB_TOKEN is required.');
	if (!repository || !tag) throw new Error('A repository and release tag are required.');
	if (!assets.length) throw new Error('At least one release asset is required.');
	const names = assets.map((path) => basename(path));
	if (new Set(names).size !== names.length) {
		throw new Error('Release asset filenames must be unique.');
	}
	for (const path of assets) {
		if (!(await stat(path)).isFile()) throw new Error(`Not a release asset file: ${path}`);
	}

	const api = `https://api.github.com/repos/${repository}`;
	async function json(url, { method = 'GET', body } = {}) {
		const response = await request(url, {
			method,
			headers: {
				Authorization: `Bearer ${token}`,
				Accept: 'application/vnd.github+json',
				'X-GitHub-Api-Version': '2022-11-28',
				...(body ? { 'Content-Type': 'application/octet-stream' } : {}),
			},
			body,
			signal: AbortSignal.timeout(10 * 60 * 1000),
		}).catch((error) => {
			throw new Error(
				`GitHub ${method} ${url} failed: ${error.message}${error.cause ? `: ${error.cause.message}` : ''}`,
				{ cause: error },
			);
		});
		if (!response.ok) {
			const details = await response.json().catch(() => ({}));
			const error = new Error(
				`GitHub ${method} ${url} failed: HTTP ${response.status}: ${details.message || response.statusText}` +
					(details.errors ? ` ${JSON.stringify(details.errors)}` : ''),
			);
			error.status = response.status;
			throw error;
		}
		return response.status === 204 ? null : response.json();
	}
	async function list(path) {
		const items = [];
		for (let page = 1; ; page++) {
			const batch = await json(`${api}/${path}?per_page=100&page=${page}`);
			items.push(...batch);
			if (batch.length < 100) return items;
		}
	}

	const release = selectDraft(await list('releases'), tag);
	for (const path of assets) {
		const name = basename(path);
		const body = await readFile(path);
		for (let attempt = 1; ; attempt++) {
			try {
				const current = await json(`${api}/releases/${release.id}`);
				if (current.draft !== true) {
					throw new Error(`Refusing to upload assets to published release ${tag}.`);
				}
				// A failed upload can leave a partial asset. Re-list on each attempt.
				const existing = await list(`releases/${release.id}/assets`);
				for (const asset of existing.filter((asset) => asset.name === name)) {
					await json(`${api}/releases/assets/${asset.id}`, { method: 'DELETE' });
				}
				await json(
					`https://uploads.github.com/repos/${repository}/releases/${release.id}/assets?name=${encodeURIComponent(name)}`,
					{ method: 'POST', body },
				);
				break;
			} catch (error) {
				if (!(error.status >= 500 && error.status < 600) || attempt === 3) throw error;
				console.warn(`${error.message}. Retrying ${name} (${attempt + 1}/3).`);
				await wait(attempt * 1000);
			}
		}
	}
	return release.html_url;
}

if (require.main === module) {
	const [tag, ...assets] = process.argv.slice(2);
	uploadRelease({ repository: REPOSITORY, tag, assets, token: process.env.GITHUB_TOKEN }).then(
		(url) => console.log(`Uploaded release assets to ${url}`),
		(error) => {
			console.error(error.message);
			process.exitCode = 1;
		},
	);
}

module.exports = { uploadRelease };

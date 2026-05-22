import { json, type RequestEvent } from '@sveltejs/kit';
import { type GitHubRelease, type GitHubReleaseAsset, GithubClient } from '$lib/GitHubClient';

const REPO_OWNER = 'automattic';
const REPO_NAME = 'harper';
const SIGNATURE_CACHE_TTL_MS = 3600 * 3000;

const signatureCache: Map<string, [string, number]> = new Map();

const responseHeaders = {
	'Cache-Control': 'no-cache',
};

const normalizeVersion = (version: string) => version.trim().replace(/^v/i, '');

const parseVersion = (version: string) => {
	const coreVersion = normalizeVersion(version).split(/[+-]/)[0];
	const parts = coreVersion.split('.').map((part) => Number.parseInt(part, 10));

	return parts.some(Number.isNaN) ? null : parts;
};

const compareVersions = (left: string, right: string) => {
	const leftParts = parseVersion(left);
	const rightParts = parseVersion(right);

	if (leftParts == null || rightParts == null) {
		return normalizeVersion(left).localeCompare(normalizeVersion(right), undefined, {
			numeric: true,
		});
	}

	const maxLength = Math.max(leftParts.length, rightParts.length);

	return (
		Array.from({ length: maxLength })
			.map((_, index) => (leftParts[index] ?? 0) - (rightParts[index] ?? 0))
			.find((diff) => diff !== 0) ?? 0
	);
};

const isCurrentVersionUpToDate = (currentVersion: string, latestVersion: string) =>
	compareVersions(currentVersion, latestVersion) >= 0;

const getAssetPrefix = (target: string, arch: string) => {
	const normalizedTarget = target.toLowerCase();
	const normalizedArch = arch.toLowerCase();
	const isMacOs = ['darwin', 'macos', 'macos-universal', 'apple-darwin'].some((targetName) =>
		normalizedTarget.includes(targetName),
	);

	if (!isMacOs) {
		return null;
	}

	if (['aarch64', 'arm64'].includes(normalizedArch)) {
		return 'harper-desktop-macos-arm64';
	}

	if (['x86_64', 'x64', 'amd64'].includes(normalizedArch)) {
		return 'harper-desktop-macos-x64';
	}

	return null;
};

const findAssetByPrefixAndSuffix = (assets: GitHubReleaseAsset[], prefix: string, suffix: string) =>
	assets.find((asset) => asset.name.startsWith(prefix) && asset.name.endsWith(suffix));

const findUpdateAssets = (release: GitHubRelease, assetPrefix: string) => {
	const archiveAsset = findAssetByPrefixAndSuffix(release.assets, assetPrefix, '.app.tar.gz');
	const signatureAsset = findAssetByPrefixAndSuffix(release.assets, assetPrefix, '.app.tar.gz.sig');

	return archiveAsset == null || signatureAsset == null ? null : { archiveAsset, signatureAsset };
};

const getSignatureFromCache = async (asset: GitHubReleaseAsset) => {
	const cacheKey = asset.browser_download_url;
	const cached = signatureCache.get(cacheKey);

	if (cached != null) {
		const [signature, expiry] = cached;

		if (expiry >= Date.now()) {
			return signature;
		}

		signatureCache.delete(cacheKey);
	}

	const resp = await fetch(asset.browser_download_url);

	if (!resp.ok) {
		return null;
	}

	const signature = (await resp.text()).trim();

	if (signature.length === 0) {
		return null;
	}

	signatureCache.set(cacheKey, [signature, Date.now() + SIGNATURE_CACHE_TTL_MS]);

	return signature;
};

const noUpdate = () => new Response(null, { status: 204, headers: responseHeaders });

export const GET = async ({ params }: RequestEvent) => {
	const { target, arch, current_version: currentVersion } = params;

	if (target == null || arch == null || currentVersion == null) {
		return noUpdate();
	}

	const assetPrefix = getAssetPrefix(target, arch);

	if (assetPrefix == null) {
		console.log(`No Harper Desktop update available for unsupported platform ${target}/${arch}.`);
		return noUpdate();
	}

	const latestRelease = await GithubClient.getLatestReleaseMetadataFromCache(REPO_OWNER, REPO_NAME);
	const latestVersion = latestRelease.tag_name;

	if (isCurrentVersionUpToDate(currentVersion, latestVersion)) {
		console.log(
			`No Harper Desktop update available for ${target}/${arch}. Current: ${currentVersion}. Latest: ${latestVersion}.`,
		);
		return noUpdate();
	}

	const updateAssets = findUpdateAssets(latestRelease, assetPrefix);

	if (updateAssets == null) {
		console.log(`No Harper Desktop updater assets found for ${assetPrefix} in ${latestVersion}.`);
		return noUpdate();
	}

	const signature = await getSignatureFromCache(updateAssets.signatureAsset);

	if (signature == null) {
		console.log(
			`Unable to fetch Harper Desktop updater signature ${updateAssets.signatureAsset.name}.`,
		);
		return noUpdate();
	}

	console.log(
		`Serving Harper Desktop update ${latestVersion} for ${target}/${arch}: ${updateAssets.archiveAsset.name}.`,
	);

	return json(
		{
			version: normalizeVersion(latestVersion),
			notes: latestRelease.body ?? latestRelease.name,
			pub_date: latestRelease.published_at,
			url: updateAssets.archiveAsset.browser_download_url,
			signature,
		},
		{ headers: responseHeaders },
	);
};

export interface GitHubReleaseAsset {
	name: string;
	browser_download_url: string;
}

export interface GitHubRelease {
	name: string;
	tag_name: string;
	body: string | null;
	published_at: string;
	assets: GitHubReleaseAsset[];
}

export class GithubClient {
	private static readonly cacheTtlMs = 3600 * 3000;
	private static versionCache: Map<string, [string, number]> = new Map();
	private static releaseCache: Map<string, [GitHubRelease, number]> = new Map();

	public static async getLatestReleaseFromCache(
		repoOwner: string,
		repoName: string,
	): Promise<string | null> {
		return await this.getFromCache(this.versionCache, `${repoOwner}/${repoName}`, () =>
			this.getLatestRelease(repoOwner, repoName),
		);
	}

	public static async getLatestReleaseMetadataFromCache(
		repoOwner: string,
		repoName: string,
	): Promise<GitHubRelease> {
		return await this.getFromCache(this.releaseCache, `${repoOwner}/${repoName}`, () =>
			this.getLatestReleaseMetadata(repoOwner, repoName),
		);
	}

	public static async getLatestReleaseAssetUrlFromCache(
		repoOwner: string,
		repoName: string,
		assetNamePattern: RegExp,
	): Promise<string | null> {
		const release = await this.getLatestReleaseMetadataFromCache(repoOwner, repoName);

		return this.findReleaseAssetUrl(release, assetNamePattern);
	}

	public static async getLatestReleaseAssetUrl(
		repoOwner: string,
		repoName: string,
		assetNamePattern: RegExp,
	): Promise<string | null> {
		const release = await this.getLatestReleaseMetadata(repoOwner, repoName);

		return this.findReleaseAssetUrl(release, assetNamePattern);
	}

	/**
	 * Return a cached value when it is still fresh, otherwise load and cache a replacement.
	 *
	 * This keeps GitHub release caching behavior consistent between lightweight version
	 * lookups and full release metadata lookups.
	 */
	private static async getFromCache<T>(
		cache: Map<string, [T, number]>,
		key: string,
		load: () => Promise<T>,
	): Promise<T> {
		const cached = cache.get(key);

		if (cached != null) {
			const [value, expiry] = cached;

			if (expiry >= Date.now()) {
				return value;
			}

			cache.delete(key);
		}

		const value = await load();
		cache.set(key, [value, Date.now() + this.cacheTtlMs]);

		return value;
	}

	public static async getLatestRelease(repoOwner: string, repoName: string): Promise<string> {
		const body = await this.getLatestReleaseMetadata(repoOwner, repoName);

		return body.tag_name;
	}

	public static async getLatestReleaseMetadata(
		repoOwner: string,
		repoName: string,
	): Promise<GitHubRelease> {
		const resp = await fetch(
			`https://api.github.com/repos/${encodeURIComponent(repoOwner)}/${encodeURIComponent(repoName)}/releases/latest`,
			{
				headers: {
					'Content-Type': 'application/json',
				},
			},
		);

		if (!resp.ok) {
			throw new Error(`Unable to get latest GitHub release: ${resp.status} ${resp.statusText}`);
		}

		return (await resp.json()) as GitHubRelease;
	}

	private static findReleaseAssetUrl(
		release: GitHubRelease,
		assetNamePattern: RegExp,
	): string | null {
		const asset = release.assets.find((asset) => assetNamePattern.test(asset.name));

		return asset?.browser_download_url ?? null;
	}
}

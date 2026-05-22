type GitHubReleaseAsset = {
	name?: string;
	browser_download_url?: string;
};

type GitHubRelease = {
	name?: string;
	assets?: GitHubReleaseAsset[];
};

export class GithubClient {
	/// Map of string -> [content, expiration time]
	private static latestReleaseCache: Map<string, [GitHubRelease, number]> = new Map();

	public static async getLatestReleaseFromCache(
		repoOwner: string,
		repoName: string,
	): Promise<string | null> {
		const release = await this.getLatestReleaseFromCacheRaw(repoOwner, repoName);

		return release.name ?? null;
	}

	public static async getLatestRelease(repoOwner: string, repoName: string): Promise<string> {
		const release = await this.getLatestReleaseRaw(repoOwner, repoName);

		if (release.name == null) {
			throw new Error(`Latest release for ${repoOwner}/${repoName} does not have a name.`);
		}

		return release.name;
	}

	public static async getLatestReleaseAssetUrlFromCache(
		repoOwner: string,
		repoName: string,
		assetNamePattern: RegExp,
	): Promise<string | null> {
		const release = await this.getLatestReleaseFromCacheRaw(repoOwner, repoName);

		return this.findReleaseAssetUrl(release, assetNamePattern);
	}

	public static async getLatestReleaseAssetUrl(
		repoOwner: string,
		repoName: string,
		assetNamePattern: RegExp,
	): Promise<string | null> {
		const release = await this.getLatestReleaseRaw(repoOwner, repoName);

		return this.findReleaseAssetUrl(release, assetNamePattern);
	}

	private static async getLatestReleaseFromCacheRaw(
		repoOwner: string,
		repoName: string,
	): Promise<GitHubRelease> {
		const key = `${repoOwner}/${repoName}`;

		const val = this.latestReleaseCache.get(key);

		if (val == null) {
			const updatedValue = await this.getLatestReleaseRaw(repoOwner, repoName);
			this.latestReleaseCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		const [value, expiry] = val;

		if (expiry - Date.now() < 0) {
			this.latestReleaseCache.delete(key);
			const updatedValue = await this.getLatestReleaseRaw(repoOwner, repoName);
			this.latestReleaseCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		return value;
	}

	private static async getLatestReleaseRaw(
		repoOwner: string,
		repoName: string,
	): Promise<GitHubRelease> {
		const resp = await fetch(
			`https://api.github.com/repos/${encodeURIComponent(repoOwner)}/${encodeURIComponent(repoName)}/releases/latest`,
			{
				headers: {
					Accept: 'application/vnd.github+json',
				},
			},
		);

		if (!resp.ok) {
			throw new Error(`Unable to fetch latest release for ${repoOwner}/${repoName}.`);
		}

		return (await resp.json()) as GitHubRelease;
	}

	private static findReleaseAssetUrl(
		release: GitHubRelease,
		assetNamePattern: RegExp,
	): string | null {
		const asset = release.assets?.find(
			(asset) => asset.name != null && assetNamePattern.test(asset.name),
		);

		return asset?.browser_download_url ?? null;
	}
}

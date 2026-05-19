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
	/// Map of string -> [content, expiration time]
	private static versionCache: Map<string, [string, number]> = new Map();
	private static releaseCache: Map<string, [GitHubRelease, number]> = new Map();

	public static async getLatestReleaseFromCache(
		repoOwner: string,
		repoName: string,
	): Promise<string | null> {
		const key = `${repoOwner}/${repoName}`;

		const val = this.versionCache.get(key);

		if (val == null) {
			const updatedValue = await this.getLatestRelease(repoOwner, repoName);
			this.versionCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		const [value, expiry] = val;

		if (expiry - Date.now() < 0) {
			this.versionCache.delete(key);
			const updatedValue = await this.getLatestRelease(repoOwner, repoName);
			this.versionCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		return value;
	}

	public static async getLatestReleaseMetadataFromCache(
		repoOwner: string,
		repoName: string,
	): Promise<GitHubRelease> {
		const key = `${repoOwner}/${repoName}`;

		const val = this.releaseCache.get(key);

		if (val == null) {
			const updatedValue = await this.getLatestReleaseMetadata(repoOwner, repoName);
			this.releaseCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		const [value, expiry] = val;

		if (expiry - Date.now() < 0) {
			this.releaseCache.delete(key);
			const updatedValue = await this.getLatestReleaseMetadata(repoOwner, repoName);
			this.releaseCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

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
}

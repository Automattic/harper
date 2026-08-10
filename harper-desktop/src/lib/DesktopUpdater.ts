import { getVersion } from '@tauri-apps/api/app';
import { check, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { Client } from '$lib/client';

const LATEST_VERSION_URL = 'https://writewithharper.com/latestversion';
const DAY_MS = 24 * 60 * 60 * 1000;

export type UpdateStatus = 'up-to-date' | 'updated' | 'error';

export interface UpdateResult {
	status: UpdateStatus;
	currentVersion?: string;
	latestVersion?: string;
	message: string;
	error?: unknown;
}

export interface UpdateOptions {
	silent?: boolean;
	onDownloadEvent?: (event: DownloadEvent) => void;
}

const normalizeVersion = (version: string) => version.trim().replace(/^v/i, '');

const shouldCheckForUpdate = (lastUpdateCheck: number | null, now = Date.now()) =>
	lastUpdateCheck == null || now - lastUpdateCheck >= DAY_MS;

export class DesktopUpdater {
	/**
	 * Fetch the latest published Harper Desktop version from the public web endpoint.
	 *
	 * This exists so settings UI can show the newest available version without invoking
	 * the full Tauri updater check/download flow.
	 */
	static async getLatestVersion(): Promise<string> {
		const resp = await fetch(LATEST_VERSION_URL, {
			headers: {
				Accept: 'text/plain',
			},
		});

		if (!resp.ok) {
			throw new Error(`Unable to get latest version: ${resp.status} ${resp.statusText}`);
		}

		return normalizeVersion(await resp.text());
	}

	/**
	 * Read the current Harper Desktop version from Tauri's bundled app metadata.
	 *
	 * This exists to keep displayed version numbers tied to the packaged app instead
	 * of hardcoded settings-page text.
	 */
	static async getCurrentVersion(): Promise<string> {
		return normalizeVersion(await getVersion());
	}

	/**
	 * Check for an available Tauri update and install it when one exists.
	 *
	 * This centralizes the updater plugin workflow so manual checks and automatic
	 * daily checks use the same behavior and return shape.
	 */
	static async updateToLatest(options: UpdateOptions = {}): Promise<UpdateResult> {
		try {
			const currentVersion = await DesktopUpdater.getCurrentVersion();
			const update = await check();

			if (update == null) {
				return {
					status: 'up-to-date',
					currentVersion,
					message: 'Harper is up to date.',
				};
			}

			await update.downloadAndInstall(options.onDownloadEvent);

			return {
				status: 'updated',
				currentVersion,
				latestVersion: normalizeVersion(update.version),
				message: 'Update installed. Restart Harper to finish.',
			};
		} catch (error) {
			if (!options.silent) {
				console.error('Unable to update Harper Desktop.', error);
			}

			return {
				status: 'error',
				message: `Unable to check for updates: ${error}`,
				error,
			};
		}
	}

	/**
	 * Run the silent daily auto-update check when the user has enabled it.
	 *
	 * This exists to keep scheduling and persisted update-check timestamps out of
	 * Svelte components while avoiding network/update checks on every app launch.
	 */
	static async maybeAutoUpdate(): Promise<UpdateResult | null> {
		const autoUpdate = await Client.getAutoUpdate();

		if (!autoUpdate) {
			return null;
		}

		const lastUpdateCheck = await Client.getLastUpdateCheck();

		if (!shouldCheckForUpdate(lastUpdateCheck)) {
			return null;
		}

		await Client.setLastUpdateCheck(Date.now());

		return await DesktopUpdater.updateToLatest({ silent: true });
	}
}

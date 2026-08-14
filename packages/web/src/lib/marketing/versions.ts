import { writable } from 'svelte/store';

export const liveVersions = writable<Record<string, string>>({
	harper: '',
	firefox: '',
	js: '',
	rust: '',
	vscode: '',
});

function isValidVersion(version: string): boolean {
	return /^\d+\.\d+\.\d+$/.test(version);
}

export function isUpToDate(harperVersion: string, integrationVersion: string): boolean | null {
	// 1. Structural/syntax invalidity
	if (!isValidVersion(harperVersion) || !isValidVersion(integrationVersion)) {
		return null;
	}

	const [hMaj, hMin, hPatch] = harperVersion.split('.').map(Number);
	const [iMaj, iMin, iPatch] = integrationVersion.split('.').map(Number);

	// 2. Logic invalidity: Integration is ahead of Harper ("Too New")
	if (iMaj > hMaj) return null;
	if (iMaj === hMaj && iMin > hMin) return null;
	if (iMaj === hMaj && iMin === hMin && iPatch > hPatch) return null;

	// 3. Exact match: Up-to-date
	if (iMaj === hMaj && iMin === hMin && iPatch === hPatch) return true;

	// 4. Fallthrough: Integration must be older
	return false;
}

export async function loadLiveVersions() {
	// 0. Harper
	(async () => {
		try {
			const ver = (await (await fetch('https://writewithharper.com/latestversion')).text()).replace(
				/^v/,
				'',
			);

			if (isValidVersion(ver)) {
				liveVersions.update((v) => ({ ...v, harper: ver }));
			}
		} catch {}
	})();

	// 1. Firefox Addon
	(async () => {
		try {
			const ver = (
				await (
					await fetch(
						'https://addons.mozilla.org/api/v5/addons/addon/private-grammar-checker-harper/',
					)
				).json()
			).current_version.version;

			if (isValidVersion(ver)) {
				liveVersions.update((v) => ({ ...v, firefox: ver }));
			}
		} catch {}
	})();

	// 2. JS Package
	(async () => {
		try {
			const ver = (await (await fetch('https://registry.npmjs.org/harper.js')).json())['dist-tags']
				.latest;

			if (isValidVersion(ver)) {
				liveVersions.update((v) => ({ ...v, js: ver }));
			}
		} catch {}
	})();

	// 3. Rust Crate
	(async () => {
		try {
			const lines = (await (await fetch('https://index.crates.io/ha/rp/harper-core')).text()).split(
				'\n',
			);
			const ver = JSON.parse(lines[lines.length - 2]).vers;

			if (isValidVersion(ver)) {
				liveVersions.update((v) => ({ ...v, rust: ver }));
			}
		} catch {}
	})();

	// 4. VS Code Extension
	(async () => {
		try {
			const res = await fetch(
				'https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery',
				{
					method: 'POST',
					headers: {
						'Content-Type': 'application/json',
						Accept: 'application/json;api-version=3.0-preview.1',
					},
					body: JSON.stringify({
						filters: [
							{
								criteria: [
									{ filterType: 8, value: 'Microsoft.VisualStudio.Code' },
									{ filterType: 7, value: 'elijah-potter.harper' },
								],
								pageNumber: 1,
								pageSize: 1,
								sortBy: 0,
								sortOrder: 0,
							},
						],
						assetTypes: [],
						flags: 194,
					}),
				},
			);
			const ver = (await res.json()).results[0].extensions[0].versions[0].version;

			if (isValidVersion(ver)) {
				liveVersions.update((v) => ({ ...v, vscode: ver }));
			}
		} catch {}
	})();
}

import { writable } from 'svelte/store';

export const liveVersions = writable<Record<string, string>>({
	firefox: '',
	js: '',
	rust: '',
	vscode: '',
});

export async function loadLiveVersions() {
	// 1. Firefox Addon
	try {
		const ver = (
			await (
				await fetch(
					'https://addons.mozilla.org/api/v5/addons/addon/private-grammar-checker-harper/',
				)
			).json()
		).current_version.version;
		liveVersions.update((v) => ({ ...v, firefox: ver }));
	} catch {}

	// 2. JS Package
	try {
		const ver = (await (await fetch('https://registry.npmjs.org/harper.js')).json())['dist-tags']
			.latest;
		liveVersions.update((v) => ({ ...v, js: ver }));
	} catch {}

	// 3. Rust Crate
	try {
		const lines = (await (await fetch('https://index.crates.io/ha/rp/harper-core')).text()).split(
			'\n',
		);
		liveVersions.update((v) => ({ ...v, rust: JSON.parse(lines[lines.length - 2]).vers }));
	} catch {}

	// 4. VS Code Extension
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
		liveVersions.update((v) => ({ ...v, vscode: ver }));
	} catch {}
}

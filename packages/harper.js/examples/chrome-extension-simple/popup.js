import { binaryInlined } from './vendor/binaryInlined.js';
import { LocalLinter } from './vendor/index.js';

const statusEl = document.getElementById('status');
const versionEl = document.getElementById('version');
const lintsEl = document.getElementById('lints');

async function main() {
	// Chrome extensions cannot load remote scripts under the default MV3 CSP.
	// Ship a local copy of the harper.js dist (see README) instead of unpkg.
	const pkg = await fetch(chrome.runtime.getURL('vendor/package.json')).then((r) => r.json());
	versionEl.textContent = pkg.version;

	const linter = new LocalLinter({ binary: binaryInlined });
	try {
		await linter.setup();
		statusEl.textContent = 'LocalLinter ready.';

		const lints = await linter.lint('This is an test');
		lintsEl.innerHTML = '';
		for (const lint of lints) {
			const item = document.createElement('li');
			item.textContent = lint.message();
			lintsEl.appendChild(item);
		}
		if (lints.length === 0) {
			const item = document.createElement('li');
			item.textContent = 'No issues found.';
			lintsEl.appendChild(item);
		}
	} finally {
		await linter.dispose();
	}
}

main().catch((err) => {
	statusEl.textContent = `Failed: ${err}`;
	console.error(err);
});

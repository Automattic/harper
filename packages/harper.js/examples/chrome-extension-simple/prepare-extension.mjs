import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const harperPkgRoot = join(root, '..', '..');
const dist = join(harperPkgRoot, 'dist');
const vendor = join(root, 'vendor');

async function main() {
	const pkg = JSON.parse(await readFile(join(harperPkgRoot, 'package.json'), 'utf8'));

	await mkdir(vendor, { recursive: true });
	await cp(dist, vendor, { recursive: true });
	await writeFile(
		join(vendor, 'package.json'),
		`${JSON.stringify({ name: pkg.name, version: pkg.version }, null, '\t')}\n`,
	);

	console.log(`Copied harper.js@${pkg.version} dist into ./vendor`);
	console.log('Load this folder as an unpacked extension in chrome://extensions');
}

main().catch((err) => {
	console.error(err);
	console.error(
		'\nBuild harper.js first (from the monorepo): pnpm --filter harper.js build\nThen re-run: node ./prepare-extension.mjs',
	);
	process.exit(1);
});

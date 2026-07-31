import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Chrome extensions cannot load remote code, so we assemble a self-contained
// `dist/` directory: the extension's own files plus a local copy of the
// `harper.js` distribution.
const root = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(root, 'dist');

const harperDist = path.dirname(fileURLToPath(import.meta.resolve('harper.js')));

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(outDir, { recursive: true });

fs.cpSync(path.join(root, 'src'), outDir, { recursive: true });
fs.cpSync(harperDist, path.join(outDir, 'vendor', 'harper.js'), {
	recursive: true,
});

// Expose the bundled `harper.js` version to the popup. The `harper.js` package
// is versioned in lockstep with `harper-core`, so this is also the version of
// the underlying grammar engine.
const { version } = JSON.parse(
	fs.readFileSync(path.join(harperDist, '..', 'package.json'), 'utf8'),
);
fs.writeFileSync(path.join(outDir, 'version.js'), `export const harperVersion = '${version}';\n`);

console.log(`Built extension into ${outDir}`);
console.log('Load it via chrome://extensions → "Load unpacked" → select dist/');

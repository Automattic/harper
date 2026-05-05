import { cp, mkdir, readdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(__dirname, '..');
const tempDir = path.join(packageDir, '.svelte-package-tmp');
const distDir = path.join(packageDir, 'dist');

async function copyDeclarations(sourceDir, targetDir) {
	const entries = await readdir(sourceDir, { withFileTypes: true });

	for (const entry of entries) {
		const sourcePath = path.join(sourceDir, entry.name);
		const targetPath = path.join(targetDir, entry.name);

		if (entry.isDirectory()) {
			await copyDeclarations(sourcePath, targetPath);
			continue;
		}

		if (!entry.name.endsWith('.d.ts') && !entry.name.endsWith('.d.ts.map')) {
			continue;
		}

		if (entry.name === 'bundle.d.ts' || entry.name === 'bundle.d.ts.map') {
			continue;
		}

		await mkdir(path.dirname(targetPath), { recursive: true });
		await cp(sourcePath, targetPath);
	}
}

await copyDeclarations(tempDir, distDir);
await rm(tempDir, { recursive: true, force: true });

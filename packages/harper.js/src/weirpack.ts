import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';

export type WeirpackManifest = Record<string, unknown>;
export type WeirpackFileMap = Map<string, string>;

export type WeirpackArchive = {
	manifest: WeirpackManifest;
	files: WeirpackFileMap;
};

const manifestFilename = 'manifest.json';

export function packWeirpackFiles(files: WeirpackFileMap): Uint8Array {
	if (!files.has(manifestFilename)) {
		throw new Error('Weirpack is missing manifest.json');
	}

	const entries: Record<string, Uint8Array> = {};
	for (const [name, content] of files.entries()) {
		entries[name] = strToU8(content);
	}

	return zipSync(entries, { level: 6 });
}

export function unpackWeirpackBytes(bytes: Uint8Array): WeirpackArchive {
	const archive = unzipSync(bytes);
	const manifestBytes = archive[manifestFilename];
	if (!manifestBytes) {
		throw new Error('Weirpack is missing manifest.json');
	}

	const manifestText = strFromU8(manifestBytes);
	const manifest = JSON.parse(manifestText) as WeirpackManifest;
	const files: WeirpackFileMap = new Map();
	files.set(manifestFilename, manifestText);

	const fileNames = Object.keys(archive);
	fileNames.sort();
	for (const name of fileNames) {
		const data = archive[name];
		if (!data || name === manifestFilename) {
			continue;
		}
		files.set(name, strFromU8(data));
	}

	return { manifest, files };
}

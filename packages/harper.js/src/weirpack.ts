import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';

export type WeirpackArchive = {
	manifest: Record<string, unknown>;
	files: Map<string, string>;
};

const manifestFilename = 'manifest.json';

/** Convert a Weirpack file system into a real Weirpack binary by compressing and serializing them into a byte array.
 *
 * For clarity on what a Weirpack is, read [the Weir documentation.](https://writewithharper.com/docs/weir#Weirpacks) */
export function packWeirpackFiles(files: Map<string, string>): Uint8Array {
	if (!files.has(manifestFilename)) {
		throw new Error('Weirpack is missing manifest.json');
	}

	const entries: Record<string, Uint8Array> = {};
	for (const [name, content] of files.entries()) {
		entries[name] = strToU8(content);
	}

	return zipSync(entries, { level: 6 });
}

/** Decompress and deserialize a Weirpack from a byte array. */
export function unpackWeirpackBytes(bytes: Uint8Array): WeirpackArchive {
	const archive = unzipSync(bytes);
	const manifestBytes = archive[manifestFilename];
	if (!manifestBytes) {
		throw new Error('Weirpack is missing manifest.json');
	}

	const manifestText = strFromU8(manifestBytes);
	const manifest = JSON.parse(manifestText);
	const files = new Map();
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

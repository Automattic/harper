import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';

export type WeirpackManifest = Record<string, unknown>;
export type WeirpackFileEntry = {
	name: string;
	content: string;
};

export type WeirpackArchive = {
	manifest: WeirpackManifest;
	rules: WeirpackFileEntry[];
};

const manifestFilename = 'manifest.json';

export function packWeirpackFiles(files: WeirpackFileEntry[]): Uint8Array {
	if (!files.some((file) => file.name === manifestFilename)) {
		throw new Error('Weirpack is missing manifest.json');
	}

	const entries: Record<string, Uint8Array> = {};
	for (const file of files) {
		entries[file.name] = strToU8(file.content);
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
	const rules: WeirpackFileEntry[] = [];

	const ruleNames = Object.keys(archive).filter((name) => name.endsWith('.weir'));
	ruleNames.sort();

	for (const name of ruleNames) {
		const data = archive[name];
		if (!data) {
			continue;
		}
		rules.push({
			name,
			content: strFromU8(data),
		});
	}

	return { manifest, rules };
}

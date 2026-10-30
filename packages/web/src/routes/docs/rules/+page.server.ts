import { LocalLinter } from 'harper.js';
import { slimBinaryInlined } from 'harper.js/slimBinaryInlined';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	// Keep the WASM in the server bundle instead of requiring an adjacent binary file.
	const linter = new LocalLinter({ binary: slimBinaryInlined });
	try {
		const [descriptions, defaultConfig] = await Promise.all([
			linter.getLintDescriptionsHTML(),
			linter.getDefaultLintConfig(),
		]);
		return { descriptions, defaultConfig };
	} finally {
		await linter.dispose();
	}
};

import { expect, test } from 'vitest';
import { slimBinary } from './binaries/slimBinary';
import LocalLinter from './LocalLinter';

// We test the 'slim' binary seperately from the main one because we utilize memoization.
// To get the module to fail when requesting abilities is does not have, we need to load it in a separate context.

test('Requesting a Typst parser with the slim binary should panic.', async () => {
	const linter = new LocalLinter({ binary: slimBinary });

	await expect(linter.lint('', { language: 'typst' })).rejects.toThrowError();
});

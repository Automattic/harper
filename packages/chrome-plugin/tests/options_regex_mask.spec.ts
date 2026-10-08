import { expect, test } from './fixtures';
import { getBackground, getStoredRegexMask } from './testUtils';

test.describe('options regex mask setting', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('persists the regex mask setting in storage', async ({ context }) => {
		const background = await getBackground(context);
		await background.evaluate(async () => {
			await chrome.storage.local.set({ regexMask: '\\b[A-Z]{1,3}\\d+\\b' });
		});

		await expect.poll(() => getStoredRegexMask(context)).toBe('\\b[A-Z]{1,3}\\d+\\b');
	});
});

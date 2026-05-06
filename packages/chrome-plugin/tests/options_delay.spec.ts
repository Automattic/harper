import { expect, test } from './fixtures';
import { getBackground, getStoredDelay } from './testUtils';

test.describe('options delay setting', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('persists the delay setting in storage', async ({ context }) => {
		const background = await getBackground(context);
		await background.evaluate(async () => {
			await chrome.storage.local.set({ delay: 750 });
		});

		await expect.poll(() => getStoredDelay(context)).toBe(750);
	});
});

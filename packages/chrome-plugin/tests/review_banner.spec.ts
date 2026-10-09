import { expect, test } from './fixtures';

test.describe('review banner', () => {
	test.skip(({ browserName }) => browserName === 'firefox', 'Review prompt disabled in Firefox');

	test('review request hidden before 14 days', async ({ context, page }) => {
		test.slow();
		const background = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
		const extensionId = background.url().split('/')[2];

		const popupUrl = `chrome-extension://${extensionId}/popup.html`;
		await page.goto(popupUrl);

		await page.getByText("Let's start writing").click();

		await expect(page.getByRole('button', { name: /^Turn Harper (on|off)/ })).toBeVisible();
		await expect(page.getByText('Enjoying Harper? A quick review helps a lot.')).toHaveCount(0);
	});

	test('review request shown after 14 days', async ({ context, page }) => {
		test.slow();
		const background = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
		const extensionId = background.url().split('/')[2];

		const popupUrl = `chrome-extension://${extensionId}/popup.html`;

		// Let startup finish writing the installation date before replacing it.
		await expect
			.poll(
				() =>
					background.evaluate(async () => {
						const { installedOn } = await chrome.storage.local.get('installedOn');
						return typeof installedOn === 'string';
					}),
				{ timeout: 30000 },
			)
			.toBe(true);

		await background.evaluate(() =>
			chrome.storage.local.set({
				installedOn: new Date(Date.now() - 15 * 86400000).toISOString(),
			}),
		);
		await page.goto(popupUrl);

		await page.getByText("Let's start writing").click();

		await expect(page.getByRole('button', { name: /^Turn Harper (on|off)/ })).toBeVisible();
		// WASM startup can delay replies; test.slow() does not extend assertion timeouts.
		await expect(page.getByText('Enjoying Harper? A quick review helps a lot.')).toHaveCount(1, {
			timeout: 30000,
		});
	});
});
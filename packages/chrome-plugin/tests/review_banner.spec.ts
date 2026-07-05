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

		await expect(page.getByText('Harper is')).toBeVisible();
		await expect(page.getByText('Would you mind giving us a review?')).toHaveCount(0);
	});

	test('review request shown after 14 days', async ({ context, page }) => {
		test.slow();
		const background = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
		const extensionId = background.url().split('/')[2];

		const popupUrl = `chrome-extension://${extensionId}/popup.html`;
		await page.goto(popupUrl);

		// 8 days
		await page.clock.install();
		await page.clock.fastForward(15 * 1000 * 60 * 60 * 24);

		await page.getByText("Let's start writing").click();

		const startTime = Date.now();
		console.log(`[TIMING] Starting wait for Harper intro text at ${startTime}`);
		await expect(page.getByText('Harper is')).toBeVisible();

		const introTime = Date.now();
		console.log(`[TIMING] Harper intro text appeared after ${introTime - startTime}ms`);

		console.log(`[TIMING] Starting wait for review banner at ${introTime}`);
		await expect(page.getByText('Would you mind giving us a review?')).toHaveCount(1);

		const endTime = Date.now();
		console.log(
			`[TIMING] Review banner appeared after ${endTime - introTime}ms (total: ${endTime - startTime}ms)`,
		);
	});
});

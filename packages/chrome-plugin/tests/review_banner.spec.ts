import { expect, test } from './fixtures';

test.describe('review banner', () => {
	test.skip(({ browserName }) => browserName === 'firefox', 'Review prompt disabled in Firefox');

	test('review request hidden before 14 days', async ({ context, page }) => {
		const background = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
		const extensionId = background.url().split('/')[2];

		const popupUrl = `chrome-extension://${extensionId}/popup.html`;
		await page.goto(popupUrl);

		await page.getByText("Let's start writing").click();

		await expect(page.getByText('Harper is')).toBeVisible();
		await expect(page.getByText('Would you mind giving us a review?')).toHaveCount(0);
	});

	test('review request shown after 14 days', async ({ context, page }) => {
		const background = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
		const extensionId = background.url().split('/')[2];

		const popupUrl = `chrome-extension://${extensionId}/popup.html`;
		await page.goto(popupUrl);

		// 8 days
		await page.clock.install();
		await page.clock.fastForward(15 * 1000 * 60 * 60 * 24);

		await page.getByText("Let's start writing").click();

		await expect(page.getByText('Harper is')).toBeVisible();
		await expect(page.getByText('Would you mind giving us a review?')).toHaveCount(1);
	});
});

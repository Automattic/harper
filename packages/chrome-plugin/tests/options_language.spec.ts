import { expect, test } from './fixtures';
import { getBackground, openExtensionPage } from './testUtils';

test.describe('language setting', () => {
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('lists the compiled languages and stores the chosen one', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');

		const select = page.getByTestId('language-select');
		await expect(select).toBeVisible({ timeout: 15000 });
		await expect(select.locator('optgroup[label="English"] option')).toHaveCount(5);
		await expect(select.locator('optgroup[label="Slovenčina"] option')).toHaveCount(1);

		const slovak = await select
			.locator('optgroup[label="Slovenčina"] option')
			.getAttribute('value');
		await select.selectOption(slovak!);

		const background = await getBackground(context);
		await expect
			.poll(() =>
				background.evaluate(async () => (await chrome.storage.local.get('dialect')).dialect),
			)
			.toBe(Number(slovak));
	});
});

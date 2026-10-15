import { expect, test } from './fixtures';
import { getBackground, openExtensionPage } from './testUtils';

test.describe('language setting', () => {
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('offers the compiled languages and stores the chosen ones', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');

		const cycle = page.getByTestId('language-cycle');
		const add = page.getByTestId('language-add');
		await expect(cycle.locator('li')).toHaveCount(1, { timeout: 15000 });
		await expect(add.locator('optgroup[label="Slovenčina"] option')).toHaveCount(1);

		const slovak = await add.locator('optgroup[label="Slovenčina"] option').getAttribute('value');
		await add.selectOption(slovak!);
		await expect(cycle.locator('li')).toHaveCount(2);

		const background = await getBackground(context);
		await expect
			.poll(() =>
				background.evaluate(
					async () => (await chrome.storage.local.get('languageCycle')).languageCycle,
				),
			)
			.toContain(Number(slovak));
	});
});

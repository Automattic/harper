import { expect, test } from './fixtures';
import { getStoredLintConfig, openExtensionPage } from './testUtils';

const STYLE_CATEGORY = 'Style and Redundancy';
const STYLE_DESCRIPTION =
	'Highlights wordy, repetitive, or overly weak phrasing that can usually be tightened up.';
const STYLE_BUTTON_TITLE = `Expand the ${STYLE_CATEGORY} category`;
const STYLE_BUTTON_COLLAPSE_TITLE = `Collapse the ${STYLE_CATEGORY} category`;
const STYLE_DROPDOWN_TITLE = `Set all rules in the ${STYLE_CATEGORY} category to their default, on, or off state.`;
const REPEATED_WORDS_TITLE = 'Set Repeated Words to its default, on, or off state.';

test.describe('structured rule settings', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('renders categories collapsed by default with category controls', async ({
		context,
		page,
	}) => {
		await openExtensionPage(context, page, 'options.html');

		await expect(page.getByTitle(STYLE_BUTTON_TITLE)).toBeVisible({ timeout: 15000 });
		await expect(page.getByTitle(STYLE_DROPDOWN_TITLE)).toBeVisible({ timeout: 15000 });
		await expect(page.locator('h3', { hasText: STYLE_CATEGORY })).toBeVisible({ timeout: 15000 });
		await expect(page.getByText(STYLE_DESCRIPTION)).toBeVisible({ timeout: 15000 });
		await expect(page.locator('h3', { hasText: 'Repeated Words' })).toHaveCount(0);
	});

	test('expands categories and indents nested rules', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');

		await page.getByTitle(STYLE_BUTTON_TITLE).click();
		await expect(page.locator('[style*="padding-left: 1.5rem"]').first()).toBeVisible();
	});

	test('search expands matching categories and reveals nested rules', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');
		await expect(page.getByTitle(STYLE_DROPDOWN_TITLE)).toBeVisible({ timeout: 15000 });

		await page.getByPlaceholder('Search for a rule…').fill('wordy');

		await expect(page.locator('h3', { hasText: STYLE_CATEGORY })).toBeVisible();
		await expect(page.getByText(STYLE_DESCRIPTION)).toBeVisible();
		await expect(page.getByTitle(REPEATED_WORDS_TITLE)).toBeVisible({ timeout: 15000 });
	});

	test('category dropdown bulk updates constituent flat rules', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');

		await page.getByTitle(STYLE_DROPDOWN_TITLE).selectOption('disable');

		await expect
			.poll(async () => {
				const config = await getStoredLintConfig(context);
				return {
					BoringWords: config.BoringWords,
					DiscourseMarkers: config.DiscourseMarkers,
					RepeatedWords: config.RepeatedWords,
				};
			})
			.toEqual({
				BoringWords: false,
				DiscourseMarkers: false,
				RepeatedWords: false,
			});

		await page.getByTitle(STYLE_DROPDOWN_TITLE).selectOption('default');

		await expect
			.poll(async () => {
				const config = await getStoredLintConfig(context);
				return {
					BoringWords: config.BoringWords,
					DiscourseMarkers: config.DiscourseMarkers,
					RepeatedWords: config.RepeatedWords,
				};
			})
			.toEqual({
				BoringWords: null,
				DiscourseMarkers: null,
				RepeatedWords: null,
			});
	});

	test('rule dropdown updates only the targeted flat rule', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');
		await expect(page.getByTitle(STYLE_DROPDOWN_TITLE)).toBeVisible({ timeout: 15000 });

		await page.getByPlaceholder('Search for a rule…').fill('Repeated Words');
		await expect(page.getByTitle(REPEATED_WORDS_TITLE)).toBeVisible({ timeout: 15000 });
		await page.getByTitle(REPEATED_WORDS_TITLE).selectOption('disable');

		await expect
			.poll(async () => {
				const config = await getStoredLintConfig(context);
				return {
					BoringWords: config.BoringWords,
					RepeatedWords: config.RepeatedWords,
				};
			})
			.toEqual({
				BoringWords: null,
				RepeatedWords: false,
			});
	});
});

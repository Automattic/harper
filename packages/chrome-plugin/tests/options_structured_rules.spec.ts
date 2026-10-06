import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { getStoredLintConfig, openExtensionPage } from './testUtils';

const STYLE_CATEGORY = 'Style and Redundancy';
const STYLE_DESCRIPTION =
	'Highlights wordy, repetitive, or overly weak phrasing that can usually be tightened up.';
const STYLE_DROPDOWN_TITLE = `Set all rules in the ${STYLE_CATEGORY} category to their default, on, or off state.`;
const REPEATED_WORDS_TITLE = 'Set Repeated Words to its default, on, or off state.';

/** The category disclosure button (name comes from the category label; state from aria-expanded). */
const styleToggle = (page: Page, expanded: boolean) =>
	page.getByRole('button', { name: STYLE_CATEGORY, exact: true, expanded });

const searchBox = (page: Page) => page.getByLabel('Search rules', { exact: true });

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

		await expect(styleToggle(page, false)).toBeVisible({ timeout: 15000 });
		await expect(page.getByTitle(STYLE_DROPDOWN_TITLE)).toBeVisible({ timeout: 15000 });
		await expect(page.getByText(STYLE_DESCRIPTION)).toBeVisible({ timeout: 15000 });
		await expect(page.locator('h3', { hasText: 'Repeated Words' })).toHaveCount(0);
	});

	test('expands categories and indents nested rules', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');

		await styleToggle(page, false).click();

		await expect(styleToggle(page, true)).toBeVisible();
		await expect(page.locator('[style*="padding-left: 1.5rem"]').first()).toBeVisible();

		await styleToggle(page, true).click();
		await expect(styleToggle(page, false)).toBeVisible();
	});

	test('search expands matching categories and reveals nested rules', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');
		await expect(page.getByTitle(STYLE_DROPDOWN_TITLE)).toBeVisible({ timeout: 15000 });

		await searchBox(page).fill('wordy');

		await expect(styleToggle(page, true)).toBeVisible();
		await expect(page.getByText(STYLE_DESCRIPTION)).toBeVisible();
		await expect(page.getByTitle(REPEATED_WORDS_TITLE)).toBeVisible({ timeout: 15000 });
	});

	test('search with no matches shows an empty state', async ({ context, page }) => {
		await openExtensionPage(context, page, 'options.html');
		await expect(page.getByTitle(STYLE_DROPDOWN_TITLE)).toBeVisible({ timeout: 15000 });

		await searchBox(page).fill('zzzz-no-such-rule');

		await expect(page.getByText('No rules match your search.')).toBeVisible();
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

		await searchBox(page).fill('Repeated Words');
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
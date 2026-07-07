import type { Page } from '@playwright/test';
import { test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	getTextarea,
	replaceEditorContent,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

/** Must be computed. */
async function getTestPageUrl(page: Page) {
	await page.goto('https://news.ycombinator.com');

	const firstLink = page.locator('.subline').first().locator('a').last();
	await firstLink.click();

	return page.url();
}

testBasicSuggestion(getTestPageUrl, getTextarea);
testCanIgnoreSuggestion(getTestPageUrl, getTextarea);
testCanBlockRuleSuggestion(getTestPageUrl, getTextarea);
testMultipleSuggestionsAndUndo(getTestPageUrl, getTextarea);

test('Hacker News wraps correctly', async ({ page }) => {
	await page.goto(await getTestPageUrl(page));

	await page.waitForTimeout(2000);
	await page.reload();

	// Needed because this element has a variable height and may offset the highlight boxes by an unknown amount.
	await page.locator('.toptext').evaluate((el) => el.remove());

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{
			x: 315.16668701171875,
			y: 115.33333587646484,
			width: 53.291656494140625,
			height: 21.333335876464844,
		},
		{
			x: 515.0104370117188,
			y: 96,
			width: 19.97918701171875,
			height: 21.333335876464844,
		},
	]);
});

test('Hacker News scrolls correctly', async ({ page }) => {
	test.slow();
	await page.goto(await getTestPageUrl(page));

	await page.waitForTimeout(2000);
	await page.reload();

	// Needed because this element has a variable height and may offset the highlight boxes by an unknown amount.
	await page.locator('.toptext').evaluate((el) => el.remove());

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the the Harper grammar checker, specifically if \n\n\n\n\n\n\n\n\n\n\n\n\nit scrolls beyo nd the height of the buffer.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{
			x: 201.9166717529297,
			y: 233.33334350585938,
			width: 46.63542175292969,
			height: 21.322906494140625,
		},
	]);
});

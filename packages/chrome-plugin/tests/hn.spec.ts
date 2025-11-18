import type { Page } from '@playwright/test';
import { test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	getTextarea,
	replaceEditorContent,
	testBasicSuggestionTextarea,
	testCanBlockRuleTextareaSuggestion,
	testCanIgnoreTextareaSuggestion,
} from './testUtils';

/** Must be computed. */
async function getTestPageUrl(page: Page) {
	await page.goto('https://news.ycombinator.com');

	const firstLink = page.locator('.subline').first().locator('a').last();
	await firstLink.click();

	return page.url();
}

testBasicSuggestionTextarea(getTestPageUrl);
testCanIgnoreTextareaSuggestion(getTestPageUrl);
testCanBlockRuleTextareaSuggestion(getTestPageUrl);

test('Hacker News wraps correctly', async ({ page }) => {
	await page.goto(await getTestPageUrl(page));

	await page.waitForTimeout(2000);
	await page.reload();

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{ x: 352.578125, y: 113, width: 63.984375, height: 19 },
		{ x: 592.484375, y: 96, width: 24, height: 19 },
	]);
});

test('Hacker News scrolls correctly', async ({ page }) => {
	await page.goto(await getTestPageUrl(page));

	await page.waitForTimeout(2000);
	await page.reload();

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the the Harper grammar checker, specifically if \n\n\n\n\n\n\n\n\n\n\n\n\nit scrolls beyo nd the height of the buffer.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [{ x: 216.625, y: 217, width: 56, height: 19 }]);
});

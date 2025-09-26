import { test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	clickHarperHighlight,
	getTextarea,
	replaceEditorContent,
	testBasicSuggestionTextarea,
	testCanIgnoreTextareaSuggestion,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/simple_textarea.html';

testBasicSuggestionTextarea(TEST_PAGE_URL);
testCanIgnoreTextareaSuggestion(TEST_PAGE_URL);

test('Wraps correctly', async ({ page }, testInfo) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	if (testInfo.project.name == 'chromium') {
		await assertHarperHighlightBoxes(page, [
			{ height: 19, width: 24, x: 241.90625, y: 27 },
			{ x: 233.90625, y: 44, width: 48, height: 19 },
			{ x: 281.90625, y: 44, width: 8, height: 19 },
			{ x: 10, y: 61, width: 8, height: 19 },
		]);
	} else {
		await assertHarperHighlightBoxes(page, [
			{ x: 218.8000030517578, y: 26, width: 21.600006103515625, height: 17 },
			{ x: 10, y: 71, width: 57.599998474121094, height: 17 },
		]);
	}
});

test('Scrolls correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the the Harper grammar checker, specifically if \n\n\n\n\n\n\n\n\n\n\n\n\nit scrolls beyo nd the height of the buffer.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [{ height: 19, width: 56, x: 97.953125, y: 63 }]);
});

test('Can dismiss with escape key', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if it is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await clickHarperHighlight(page);

	await page.locator('.harper-container').waitFor({ state: 'visible' });

	await page.keyboard.press('Escape');

	await page.locator('.harper-container').waitFor({ state: 'hidden' });
});

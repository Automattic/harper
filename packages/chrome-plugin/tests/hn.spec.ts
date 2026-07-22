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

const TEST_PAGE_URL = 'http://localhost:8081/hn.html';

testBasicSuggestion(TEST_PAGE_URL, getTextarea);
testCanIgnoreSuggestion(TEST_PAGE_URL, getTextarea);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getTextarea);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getTextarea);

test('Hacker News wraps correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await page.waitForTimeout(2000);
	await page.reload();

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(12000);

	await assertHarperHighlightBoxes(page, [
		[
			{ x: 353.34375, y: 111, width: 64.203125, height: 17 },
			{ x: 594.0625, y: 96, width: 24.09375, height: 17 },
		],
		[
			{ x: 354.26666259765625, y: 115, width: 64.13333129882812, height: 19 },
			{ x: 594.7666625976562, y: 98, width: 24.04998779296875, height: 19 },
		],
	]);
});

test('Hacker News scrolls correctly', async ({ page }) => {
	test.slow();
	await page.goto(TEST_PAGE_URL);

	await page.waitForTimeout(2000);
	await page.reload();

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the the Harper grammar checker, specifically if \n\n\n\n\n\n\n\n\n\n\n\n\nit scrolls beyo nd the height of the buffer.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		[{ x: 216.9375, y: 203, width: 56.171875, height: 17 }],
		[{ x: 217.98333740234375, y: 221, width: 56.116668701171875, height: 19 }],
	]);
});

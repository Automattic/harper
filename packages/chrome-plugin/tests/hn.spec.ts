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
			{ x: 315.25, y: 115, width: 53.328125, height: 21 },
			{ x: 515.171875, y: 96, width: 20.015625, height: 21 },
		],
		[
			{ x: 304.66668701171875, y: 121, width: 53.333343505859375, height: 22 },
			{ x: 504.66668701171875, y: 101, width: 20, height: 22 },
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
		[{ x: 201.96875, y: 231, width: 46.65625, height: 21 }],
		[{ x: 191.3333282470703, y: 245, width: 46.66667175292969, height: 22 }],
	]);
});

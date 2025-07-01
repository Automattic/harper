import { test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	getTextarea,
	replaceEditorContent,
	testBasicSuggestionTextarea,
	testCanIgnoreTextareaSuggestion,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/simple_textarea.html';

testBasicSuggestionTextarea(TEST_PAGE_URL);
testCanIgnoreTextareaSuggestion(TEST_PAGE_URL);

test('Wraps correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{ height: 19, width: 24, x: 241.90625, y: 27 },
		{ height: 19, width: 48, x: 233.90625, y: 44 },
		{ height: 19, width: 8, x: 281.90625, y: 44 },
		{ height: 19, width: 8, x: 10, y: 61 },
	]);
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

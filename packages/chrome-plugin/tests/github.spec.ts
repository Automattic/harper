import { test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	getTextarea,
	replaceEditorContent,
	testBasicSuggestionTextarea,
	testCanBlockRuleTextareaSuggestion,
	testCanIgnoreTextareaSuggestion,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/github_textarea.html';

testBasicSuggestionTextarea(TEST_PAGE_URL);
testCanIgnoreTextareaSuggestion(TEST_PAGE_URL);
testCanBlockRuleTextareaSuggestion(TEST_PAGE_URL);

test('Wraps correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await page.waitForTimeout(2000);
	await page.reload();

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{ x: 260.234375, y: 103, width: 67.21875, height: 18 },
		{ x: 512.28125, y: 63, width: 25.21875, height: 18 },
	]);
});

test('Scrolls correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await page.waitForTimeout(2000);
	await page.reload();

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the the Harper grammar checker, specifically if \n\n\n\n\n\n\n\n\n\n\n\n\nit scrolls beyo nd the height of the buffer.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [{ width: 58.828125, x: 117.40625, y: 161, height: 18 }]);
});

import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getHarperHighlights,
	getSlateEditor,
	randomString,
	replaceEditorContent,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://slatejs.org';

test('Can apply basic suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const slate = getSlateEditor(page);
	await replaceEditorContent(slate, 'This is an test');

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(3000);

	await expect(slate).toContainText('This is a test');

	// Verify editor state is preserved: arrow keys and backspace must work.
	// Position cursor before 's' in 'test', then backspace to delete 'e'.
	await page.press('body', 'End');
	await page.press('body', 'ArrowLeft');
	await page.press('body', 'ArrowLeft');
	await page.press('body', 'Backspace');
	await expect(slate).toContainText('This is a tst');

	// Verify typing still works.
	await slate.pressSequentially('e');
	await expect(slate).toContainText('This is a test');
});

testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getSlateEditor);

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);
	const slate = getSlateEditor(page);

	const cacheSalt = randomString(5);
	await replaceEditorContent(slate, cacheSalt);

	await page.waitForTimeout(3000);

	const opened = await clickHarperHighlight(page);
	expect(opened).toBe(true);
	await page.getByTitle('Ignore this lint').click();

	await expect(getHarperHighlights(page)).toHaveCount(0);

	// Nothing should change.
	await expect(slate).toContainText(cacheSalt);
	expect(await clickHarperHighlight(page)).toBe(false);
});

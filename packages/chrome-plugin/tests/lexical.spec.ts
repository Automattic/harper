import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getHarperHighlights,
	getLexicalEditor,
	randomString,
	replaceEditorContent,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://playground.lexical.dev/';

test('Can apply basic suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const lexical = getLexicalEditor(page);
	await replaceEditorContent(lexical, 'This is an test');

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(3000);

	await expect(lexical).toContainText('This is a test');

	// Verify editor state is preserved: arrow keys and backspace must work.
	await lexical.press('End');
	await lexical.press('ArrowLeft');
	await lexical.press('ArrowLeft');
	await lexical.press('Backspace');
	await expect(lexical).toContainText('This is a tst');

	// Verify typing still works.
	await lexical.pressSequentially('e');
	await expect(lexical).toContainText('This is a test');
});

testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getLexicalEditor);

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);
	const lexical = getLexicalEditor(page);

	const cacheSalt = randomString(5);
	await replaceEditorContent(lexical, cacheSalt);

	await page.waitForTimeout(3000);

	const opened = await clickHarperHighlight(page);
	expect(opened).toBe(true);
	await page.getByTitle('Ignore this lint').click();

	await expect(getHarperHighlights(page)).toHaveCount(0);

	// Nothing should change.
	await expect(lexical).toContainText(cacheSalt);
	expect(await clickHarperHighlight(page)).toBe(false);
});

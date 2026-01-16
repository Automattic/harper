import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getDraftEditor,
	getHarperHighlights,
	randomString,
	replaceEditorContent,
} from './testUtils';

const TEST_PAGE_URL = 'https://draftjs.org/';

test('Can apply basic suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const draft = getDraftEditor(page);
	await draft.scrollIntoViewIfNeeded();
	await draft.click();
	await replaceEditorContent(draft, 'This is an test');

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(3000);

	expect(draft).toContainText('This is a test');

	// Verify typing still works after applying suggestion.
	await draft.press('End');
	await draft.pressSequentially(" of Harper's grammar checking.");
	expect(draft).toContainText("This is a test of Harper's grammar checking.");
});

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);
	const draft = getDraftEditor(page);

	await draft.scrollIntoViewIfNeeded();
	await draft.click();

	const cacheSalt = randomString(5);
	await replaceEditorContent(draft, cacheSalt);

	await page.waitForTimeout(3000);

	const opened = await clickHarperHighlight(page);
	expect(opened).toBe(true);
	await page.getByTitle('Ignore this lint').click();

	await expect(getHarperHighlights(page)).toHaveCount(0);

	// Nothing should change.
	expect(draft).toContainText(cacheSalt);
	expect(await clickHarperHighlight(page)).toBe(false);
});

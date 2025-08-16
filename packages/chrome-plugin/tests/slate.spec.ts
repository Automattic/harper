import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getSlateEditor,
	randomString,
	replaceEditorContent,
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

	expect(slate).toContainText('This is a test');

	// Slate has be known to revert changes after typing some more.
	await slate.pressSequentially(" of Harper's grammar checking.");
	expect(slate).toContainText("This is a test of Harper's grammar checking.");
});

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);
	const slate = getSlateEditor(page);

	const cacheSalt = randomString(5);
	await replaceEditorContent(slate, cacheSalt);

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Ignore this lint').click();

	await page.waitForTimeout(3000);

	// Nothing should change.
	expect(slate).toContainText(cacheSalt);
	expect(await clickHarperHighlight(page)).toBe(false);
});

import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getLexicalEditor,
	randomString,
	replaceEditorContent,
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
	await lexical.press('Control+ArrowDown');

	await lexical.pressSequentially(" of Harper's grammar checking.");
	await expect(lexical).toContainText("This is a test of Harper's grammar checking.");
});

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);
	const lexical = getLexicalEditor(page);

	const cacheSalt = randomString(5);
	await replaceEditorContent(lexical, cacheSalt);

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Ignore this lint').click();

	await page.waitForTimeout(3000);

	// Nothing should change.
	expect(lexical).toContainText(cacheSalt);
	expect(await clickHarperHighlight(page)).toBe(false);
});

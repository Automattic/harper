import { expect, test } from './fixtures';
import { clickHarperHighlight } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/ckeditor_simple.html';

test('Keeps CKEditor edits synchronized after applying a suggestion', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await page.waitForTimeout(2000);
	await page.reload();

	await page.waitForTimeout(6000);

	const opened = await clickHarperHighlight(page);
	expect(opened).toBe(true);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(2000);

	const editor = page.locator('.ck-editor__editable');
	await expect(editor).toContainText('This is a test');

	await editor.click();
	await editor.evaluate((el) => {
		const range = document.createRange();
		range.selectNodeContents(el);
		range.collapse(false);
		const sel = window.getSelection();
		sel?.removeAllRanges();
		sel?.addRange(range);
	});
	await editor.pressSequentially('!');

	await page.waitForTimeout(1000);
	await expect(editor).toContainText('This is a test!');
});

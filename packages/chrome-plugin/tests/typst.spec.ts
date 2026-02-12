import { expect, test } from './fixtures';
import { clickHarperHighlight, getHarperHighlights } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/typst_codemirror.html';

test('Typst CodeMirror editor can apply a suggestion', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = page.locator('#typst-editor');
	const mirror = page.locator('#typst-mirror');

	await expect(editor).toContainText('This is an test');
	await expect(getHarperHighlights(page)).toHaveCount(1, { timeout: 15000 });

	expect(await clickHarperHighlight(page)).toBe(true);
	await page.getByTitle('Replace with "a"').click();

	await expect(editor).toContainText('This is a test');
	await expect(mirror).toHaveText('This is a test');

	await editor.press('End');
	await editor.pressSequentially('.');
	await expect(mirror).toHaveText('This is a test.');
});

import { expect, test } from './fixtures';
import { clickHarperHighlight, getTextarea, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/strict_csp_textarea.html';

test('Styles the suggestion popup on a page with a strict style-src', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await replaceEditorContent(getTextarea(page), 'This is an test');
	expect(await clickHarperHighlight(page)).toBe(true);

	// Without its stylesheet the popup is laid out in the page flow instead of floating.
	const popup = page.locator('.harper-popup');
	await expect(popup).toHaveCSS('position', 'fixed');
	await expect(page.locator('.harper-container')).toHaveCSS(
		'background-color',
		'rgb(255, 255, 255)',
	);
});

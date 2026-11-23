import { expect, test } from './fixtures';
import { getHarperHighlights } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/nested_editable.html';

test('Lints an editable block inside a non-editable wrapper.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await expect(getHarperHighlights(page).first()).toBeVisible({ timeout: 24000 });
});

import { test } from './fixtures';
import { assertHarperHighlightBoxes } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/simple_inputs_disabled.html';

test('Ignores disabled and readonly inputs', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await page.waitForTimeout(6000);

	// All inputs on this page are disabled or read-only, so no lint boxes should be drawn.
	await assertHarperHighlightBoxes(page, []);
});

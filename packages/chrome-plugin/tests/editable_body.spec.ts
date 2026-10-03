import { expect, test } from './fixtures';
import { getHarperHighlights } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/editable_body.html';

test('Keeps render boxes out of an editable body.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	await expect(getHarperHighlights(page).first()).toBeVisible({ timeout: 24000 });

	// Inside the body they would be part of the edited document (e.g. serialized into an e-mail).
	const renderBoxesInBody = await page.evaluate(
		() => document.body.querySelectorAll('harper-render-box').length,
	);
	expect(renderBoxesInBody).toBe(0);
});

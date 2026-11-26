import { expect, test } from './fixtures';
import { assertPageHasNHighlights } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/simple_inputs_spellcheck.html';

test('Follows the effective spellcheck state of single-line inputs', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	// The browser decides whether a single-line input without a spellcheck attribute is checked.
	const defaultOn = await page
		.locator('input')
		.first()
		.evaluate((el) => (el as HTMLInputElement).spellcheck);

	await page.waitForTimeout(6000);

	// Inputs with spellcheck disabled (directly or inherited) and non-text inputs are never linted.
	await assertPageHasNHighlights(page, defaultOn ? 1 : 0);
	expect(typeof defaultOn).toBe('boolean');
});

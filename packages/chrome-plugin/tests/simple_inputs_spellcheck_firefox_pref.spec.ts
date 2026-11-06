import { test } from './fixtures';
import { assertPageHasNHighlights } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/simple_inputs_spellcheck.html';

// Firefox checks single-line fields too when `layout.spellcheckDefault` is 2.
test.use({ launchOptions: { firefoxUserPrefs: { 'layout.spellcheckDefault': 2 } } });

test('Lints single-line inputs when Firefox spellchecks them by default', async ({
	page,
	browserName,
}) => {
	test.skip(browserName !== 'firefox', 'Firefox preference');

	await page.goto(TEST_PAGE_URL);
	await page.waitForTimeout(6000);

	await assertPageHasNHighlights(page, 1);
});

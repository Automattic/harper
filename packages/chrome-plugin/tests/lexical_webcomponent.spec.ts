import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getHarperHighlights,
	getLexicalEditor,
	replaceEditorContent,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/lexical_webcomponent.html';

test.describe('Lexical webcomponent regression', () => {
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox extension build lacks background scripts',
	);
	test('Applying a suggestion does not duplicate text', async ({ page }) => {
		await page.goto(TEST_PAGE_URL);

		const lexical = getLexicalEditor(page);
		const mirror = page.locator('#lexical-mirror');
		const initialText = 'This is an test. This is an test again.';
		await replaceEditorContent(lexical, initialText);

		await page.waitForTimeout(6000);
		await expect(mirror).toHaveText(initialText);

		await clickHarperHighlight(page);
		await page.getByTitle('Replace with "a"').click();

		await page.waitForTimeout(3000);
		const afterFirst = 'This is a test. This is an test again.';
		await expect(lexical).toHaveText(afterFirst);
		await expect(mirror).toHaveText(afterFirst);
		await expect(getHarperHighlights(page)).toHaveCount(1);

		await clickHarperHighlight(page);
		await page.getByTitle('Replace with "a"').click();

		await page.waitForTimeout(3000);
		const finalText = 'This is a test. This is a test again.';
		await expect(lexical).toHaveText(finalText);
		await expect(mirror).toHaveText(finalText);
		await expect(getHarperHighlights(page)).toHaveCount(0);

		const lexicalText = (await lexical.textContent()) ?? '';
		const mirrorText = (await mirror.textContent()) ?? '';
		expect(lexicalText).toBe(mirrorText);
	});
});

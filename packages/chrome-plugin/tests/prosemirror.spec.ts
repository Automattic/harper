import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getProseMirrorEditor,
	randomString,
	replaceEditorContent,
} from './testUtils';

const TEST_PAGE_URL = 'https://prosemirror.net/';

test('Can apply basic suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const pm = getProseMirrorEditor(page);
	await replaceEditorContent(pm, 'This is an test');

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(3000);

	await expect(pm).toContainText('This is a test');
	await pm.press('Control+ArrowDown');

	await pm.pressSequentially(' of Harper’s grammar checking.');
	await expect(pm).toContainText('This is a test of Harper’s grammar checking.');
});

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);
	const pm = getProseMirrorEditor(page);

	const cacheSalt = randomString(5);
	await replaceEditorContent(pm, cacheSalt);

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Ignore this lint').click();

	await page.waitForTimeout(3000);

	// Nothing should change.
	expect(pm).toContainText(cacheSalt);
	expect(await clickHarperHighlight(page)).toBe(false);
});

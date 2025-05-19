import type { Locator, Page } from '@playwright/test';
import type { Box } from '../src/Box';
import { expect, test } from './fixtures';
import { clickHarperHighlight, getHarperHighlights, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/github_textarea.html';

/** Grab the first `<textarea />` on a page. */
function getTextarea(page: Page): Locator {
	return page.locator('textarea');
}

test('Can apply basic suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(editor, 'This is an test');

	await page.waitForTimeout(6000);

	await clickHarperHighlight(page);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(3000);

	expect(editor).toHaveValue('This is a test');
});

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(editor, 'This is an test');

	await page.waitForTimeout(6000);

	await clickHarperHighlight(page);
	await page.getByTitle('Ignore this lint').click();

	await page.waitForTimeout(3000);

	// Nothing should change.
	expect(editor).toHaveValue('This is an test');
	expect(await clickHarperHighlight(page)).toBe(false);
});

test('Wraps correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{ height: 18, width: 25.21875, x: 512.28125, y: 63 },
		{ height: 18, width: 67.21875, x: 260.234375, y: 103 },
	]);
});

async function assertHarperHighlightBoxes(page: Page, boxes: Box[]): Promise<void> {
	const highlights = getHarperHighlights(page);

	expect(await highlights.count()).toBe(boxes.length);

	for (let i = 0; i < (await highlights.count()); i++) {
		expect(await highlights.nth(i).boundingBox()).toStrictEqual(boxes[i]);
	}
}

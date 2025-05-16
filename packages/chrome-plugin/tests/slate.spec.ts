import type { Locator, Page } from '@playwright/test';
import { expect, test } from './fixtures';

/** Locates the first Harper highlight on the page and clicks it.
 * It should result in the popup opening.
 * Returns whether the highlight was found. */
async function clickHarperHighlight(page: Page): Promise<boolean> {
	const highlight = page.locator('#harper-highlight');

	if ((await highlight.count()) == 0) {
		return false;
	}

	const box = await highlight.boundingBox();

	if (box == null) {
		return false;
	}

	// Locate the center of the element.
	const cx = box.x + box.width / 2;
	const cy = box.y + box.height / 2;

	await page.mouse.click(cx, cy);
	return true;
}

function getSlateEditor(page: Page): Locator {
	return page.locator('[data-slate-editor="true"]');
}

async function replaceSlateContent(page: Page, text: string) {
	const slateEditor = getSlateEditor(page);

	await slateEditor.selectText();
	await slateEditor.press('Backspace');
	await slateEditor.pressSequentially(text);
}

test('Can apply basic suggestion.', async ({ page }) => {
	await page.goto('https://slatejs.org');

	await replaceSlateContent(page, 'This is an test');

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Replace with "a"').click();

	await page.waitForTimeout(3000);

	const slateEditor = getSlateEditor(page);
	expect(slateEditor).toContainText('This is a test');

	// Slate has be known to revert changes after typing some more.
	await slateEditor.pressSequentially(" of Harper's grammar checking.");
	expect(slateEditor).toContainText("This is a test of Harper's grammar checking.");
});

test('Can ignore suggestion.', async ({ page }) => {
	await page.goto('https://slatejs.org');

	await replaceSlateContent(page, 'This is an test.');

	await page.waitForTimeout(3000);

	await clickHarperHighlight(page);
	await page.getByTitle('Ignore this lint').click();

	await page.waitForTimeout(3000);

	// Nothing should change.
	expect(getSlateEditor(page)).toContainText('This is an test');
	expect(await clickHarperHighlight(page)).toBe(false);
});

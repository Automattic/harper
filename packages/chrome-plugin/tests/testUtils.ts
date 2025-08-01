import type { Locator, Page } from '@playwright/test';
import type { Box } from '../src/Box';
import { expect, test } from './fixtures';

export function randomString(length: number): string {
	const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
	let result = '';
	for (let i = 0; i < length; i++) {
		result += chars.charAt(Math.floor(Math.random() * chars.length));
	}
	return result;
}

/** Locate the [`Slate`](https://www.slatejs.org/examples/richtext) editor on the page.  */
export function getSlateEditor(page: Page): Locator {
	return page.locator('[data-slate-editor="true"]');
}

/** Locate the [`Lexical`](https://lexical.dev/) editor on the page.  */
export function getLexicalEditor(page: Page): Locator {
	return page.locator('[data-lexical-editor="true"]');
}

/** Locate the ProseMirror editor on the page.  */
export function getProseMirrorEditor(page: Page): Locator {
	return page.locator('.ProseMirror');
}

/** Replace the content of a text editor. */
export async function replaceEditorContent(editorEl: Locator, text: string) {
	await editorEl.selectText();
	await editorEl.press('Backspace');
	await editorEl.pressSequentially(text);
}

/** Locate the Harper highlights on a page. */
export function getHarperHighlights(page: Page): Locator {
	return page.locator('#harper-highlight');
}

/** Locates the first Harper highlight on the page and clicks it.
 * It should result in the popup opening.
 * Returns whether the highlight was found. */
export async function clickHarperHighlight(page: Page): Promise<boolean> {
	const highlights = getHarperHighlights(page);

	if ((await highlights.count()) == 0) {
		return false;
	}

	const box = await highlights.first().boundingBox();

	if (box == null) {
		return false;
	}

	// Locate the center of the element.
	const cx = box.x + box.width / 2;
	const cy = box.y + box.height / 2;

	await page.mouse.click(cx, cy);
	return true;
}

/** Grab the first `<textarea />` on a page. */
export function getTextarea(page: Page): Locator {
	return page.locator('textarea');
}

export async function testBasicSuggestionTextarea(testPageUrl: string) {
	test('Can apply basic suggestion.', async ({ page, context }) => {
		await page.goto(testPageUrl);

		await page.waitForTimeout(2000);
		await page.reload();

		const editor = getTextarea(page);
		await replaceEditorContent(editor, 'This is an test');

		await page.waitForTimeout(6000);

		await clickHarperHighlight(page);
		await page.getByTitle('Replace with "a"').click();

		await page.waitForTimeout(3000);

		expect(editor).toHaveValue('This is a test');
	});
}

export async function testCanIgnoreTextareaSuggestion(testPageUrl: string) {
	test('Can ignore suggestion.', async ({ page }) => {
		await page.goto(testPageUrl);

		await page.waitForTimeout(2000);
		await page.reload();

		const editor = getTextarea(page);

		const cacheSalt = randomString(5);
		await replaceEditorContent(editor, cacheSalt);

		await page.waitForTimeout(6000);

		await clickHarperHighlight(page);
		await page.getByTitle('Ignore this lint').click();

		await page.waitForTimeout(3000);

		// Nothing should change.
		expect(editor).toHaveValue(cacheSalt);
		expect(await clickHarperHighlight(page)).toBe(false);
	});
}

export async function assertHarperHighlightBoxes(page: Page, boxes: Box[]): Promise<void> {
	const highlights = getHarperHighlights(page);
	expect(await highlights.count()).toBe(boxes.length);

	for (let i = 0; i < (await highlights.count()); i++) {
		const box = await highlights.nth(i).boundingBox();

		console.log(`Expected: ${JSON.stringify(boxes[i])}`);
		console.log(`Got: ${JSON.stringify(box)}`);

		assertBoxesClose(box, boxes[i]);
	}
}

/** An assertion that checks to ensure that two boxes are _approximately_ equal.
 * Leaves wiggle room for floating point error. */
export function assertBoxesClose(a: Box, b: Box) {
	assertClose(a.x, b.x);
	assertClose(a.y, b.y);
	assertClose(a.width, b.width);
	assertClose(a.height, b.height);
}

function assertClose(actual: number, expected: number) {
	expect(Math.abs(actual - expected)).toBeLessThanOrEqual(9);
}

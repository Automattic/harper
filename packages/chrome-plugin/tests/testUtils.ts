import type { Locator, Page } from '@playwright/test';
import type { Box } from 'lint-framework';
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

/** Locate the Draft.js editor on the page (targets #rich-example on draftjs.org). */
export function getDraftEditor(page: Page): Locator {
	return page.locator('#rich-example .public-DraftEditor-content');
}

/** Replace the content of a text editor. Handles newlines by pressing Enter. */
export async function replaceEditorContent(editorEl: Locator, text: string) {
	await editorEl.selectText();
	await editorEl.press('Backspace');

	const lines = text.split('\n');
	for (let i = 0; i < lines.length; i++) {
		await editorEl.pressSequentially(lines[i]);
		if (i < lines.length - 1) {
			await editorEl.press('Enter');
		}
	}
}

/** Locate the Harper highlights on a page. */
export function getHarperHighlights(page: Page): Locator {
	return page.locator('#harper-highlight');
}

export async function assertLocatorIsFocused(page: Page, loc: Locator) {
	await assertLocatorsResolveEqually(page, loc, page.locator(':focus'));
}

/**  Checks that the two provided locators resolve to the same element. */
export async function assertLocatorsResolveEqually(page: Page, a: Locator, b: Locator) {
	const areSame = await page.evaluate(
		([a, b]) => a === b,
		[await a.elementHandle(), await b.elementHandle()],
	);

	expect(areSame).toBe(true);
}

/** Locates the first Harper highlight on the page and clicks it.
 * It should result in the popup opening.
 * Returns whether the highlight was found. */
export async function clickHarperHighlight(page: Page): Promise<boolean> {
	const highlights = getHarperHighlights(page);

	// Wait briefly for at least one highlight to appear.
	// If none appear within a reasonable time, return false.
	try {
		await highlights.first().waitFor({ state: 'visible', timeout: 5000 });
	} catch {
		return false;
	}

	const box = await highlights.first().boundingBox();
	if (box == null) return false;

	// Locate the center of the element and click to open the popup.
	const cx = box.x + box.width / 2;
	const cy = box.y + box.height / 2;
	await page.mouse.click(cx, cy);
	return true;
}

/** Grab the first `<textarea />` on a page. */
export function getTextarea(page: Page): Locator {
	return page.locator('textarea');
}

/** A string or function that resolves to a test page. */
export type TestPageUrlProvider = string | ((page: Page) => Promise<string>);

/** A function that returns an editor locator. */
export type EditorLocatorProvider = (page: Page) => Locator;

async function resolveTestPage(prov: TestPageUrlProvider, page: Page): Promise<string> {
	if (typeof prov === 'string') {
		return prov;
	} else {
		return await prov(page);
	}
}

export async function testBasicSuggestionTextarea(testPageUrl: TestPageUrlProvider) {
	test('Can apply basic suggestion.', async ({ page }) => {
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		await page.waitForTimeout(2000);
		await page.reload();

		const editor = getTextarea(page);
		await replaceEditorContent(editor, 'This is an test');

		await page.waitForTimeout(6000);

		await clickHarperHighlight(page);
		await page.getByTitle('Replace with "a"').click();

		await page.waitForTimeout(3000);

		expect(editor).toHaveValue('This is a test');
		await assertLocatorIsFocused(page, editor);
	});
}

export async function testCanIgnoreTextareaSuggestion(testPageUrl: TestPageUrlProvider) {
	test('Can ignore suggestion.', async ({ page }) => {
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		await page.waitForTimeout(2000);
		await page.reload();

		const editor = getTextarea(page);

		const cacheSalt = randomString(5);
		await replaceEditorContent(editor, cacheSalt);

		await page.waitForTimeout(6000);

		// Open the popup for the first highlight and click Ignore.
		const opened = await clickHarperHighlight(page);
		expect(opened).toBe(true);
		await page.getByTitle('Ignore this lint').click();

		// Wait for highlights to disappear after ignoring.
		await expect(getHarperHighlights(page)).toHaveCount(0);

		// Nothing should change.
		expect(editor).toHaveValue(cacheSalt);
		expect(await clickHarperHighlight(page)).toBe(false);
		await assertLocatorIsFocused(page, editor);
	});
}

export async function testCanBlockRuleTextareaSuggestion(testPageUrl: TestPageUrlProvider) {
	test('Can hide with rule block button', async ({ page }) => {
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		const editor = getTextarea(page);
		await replaceEditorContent(editor, 'This is an test.');

		await page.waitForTimeout(6000);

		await clickHarperHighlight(page);

		await page.getByTitle('Disable the AnA rule').click();

		await page.waitForTimeout(1000);

		await assertHarperHighlightBoxes(page, []);
		await assertLocatorIsFocused(page, editor);
	});
}

/** Get highlight bounding boxes sorted by visual position (top to bottom, left to right). */
async function getSortedHighlightBoxes(page: Page) {
	const highlights = getHarperHighlights(page);
	const count = await highlights.count();
	const boxes: NonNullable<Awaited<ReturnType<Locator['boundingBox']>>>[] = [];

	for (let i = 0; i < count; i++) {
		const box = await highlights.nth(i).boundingBox();
		if (box) {
			boxes.push(box);
		}
	}

	boxes.sort((a, b) => (Math.abs(a.y - b.y) > 5 ? a.y - b.y : a.x - b.x));

	return boxes;
}

/** Test multiline suggestion replacement and undo. */
export async function testMultipleSuggestionsAndUndo(
	testPageUrl: TestPageUrlProvider,
	getEditor: EditorLocatorProvider,
	setup?: (editor: Locator) => Promise<void>,
) {
	test('Multiple suggestions and undo.', async ({ page }) => {
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		const editor = getEditor(page);
		if (setup) {
			await setup(editor);
		}
		await replaceEditorContent(editor, 'The first tset.\nThe second tset.\nThe third tset.');

		await page.waitForTimeout(3000);

		const highlights = getHarperHighlights(page);
		await expect(highlights).toHaveCount(3);

		// Get highlights sorted by visual position and click on the middle one
		const sortedBoxes = await getSortedHighlightBoxes(page);
		expect(sortedBoxes.length).toBe(3);
		const box = sortedBoxes[1];
		await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);

		// Move cursor away to test whether it handles race condition
		await editor.press('End');

		await page.getByTitle('Replace with "test"').click();
		await page.waitForTimeout(500);

		// Verify only second "tset" was corrected
		await expect(editor).toContainText('first tset');
		await expect(editor).toContainText('second test');
		await expect(editor).toContainText('third tset');

		// Undo
		await editor.press('Control+z');
		await page.waitForTimeout(300);
		await expect(editor).toContainText('The second tset');
	});
}

export async function assertHarperHighlightBoxes(page: Page, boxes: Box[]): Promise<void> {
	const highlights = getHarperHighlights(page);
	await expect(highlights).toHaveCount(boxes.length);

	for (let i = 0; i < (await highlights.count()); i++) {
		const box = await highlights.nth(i).boundingBox();
		expect(box).not.toBeNull();

		console.log(`Expected: ${JSON.stringify(boxes[i])}`);
		console.log(`Got: ${JSON.stringify(box)}`);

		assertBoxesClose(box!, boxes[i]);
	}
}

/** Create a test to assert that a page has a certain number highlights.
 * Wraps `assertPageHasNHighlights` */
export async function testPageHasNHighlights(testPageUrl: TestPageUrlProvider, n: number) {
	test(`Page has ${n} highlights`, async ({ page }) => {
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		await page.waitForTimeout(6000);

		assertPageHasNHighlights(page, n);
	});
}

/** Assert that the page has a specific number of highlights.
 * Useful for making sure certain patterns are ignored. */
export async function assertPageHasNHighlights(page: Page, n: number) {
	const highlights = getHarperHighlights(page);
	expect(await highlights.count()).toBe(n);
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
	expect(Math.abs(actual - expected)).toBeLessThanOrEqual(15);
}

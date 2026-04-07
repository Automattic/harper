import type { BrowserContext, Locator, Page } from '@playwright/test';
import type { LintConfig } from 'harper.js';
import type { Box } from 'lint-framework';
import { expect, test } from './fixtures';

type ScreenPoint = {
	x: number;
	y: number;
};

export function randomString(length: number): string {
	const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
	let result = '';
	for (let i = 0; i < length; i++) {
		result += chars.charAt(Math.floor(Math.random() * chars.length));
	}
	return result;
}

export async function getBackground(context: BrowserContext) {
	return (
		context.serviceWorkers()[0] ??
		context.backgroundPages()[0] ??
		(await context.waitForEvent('serviceworker'))
	);
}

export async function getExtensionId(context: BrowserContext): Promise<string> {
	const background = await getBackground(context);
	return background.url().split('/')[2];
}

export async function openExtensionPage(
	context: BrowserContext,
	page: Page,
	path: 'popup.html' | 'options.html',
) {
	const extensionId = await getExtensionId(context);
	await page.goto(`chrome-extension://${extensionId}/${path}`);
}

export async function getStoredLintConfig(context: BrowserContext): Promise<LintConfig> {
	const background = await getBackground(context);
	return await background.evaluate(async () => {
		const value = await chrome.storage.local.get('lintConfig');
		return JSON.parse(value.lintConfig ?? '{}');
	});
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

/**
 * Wait for the first Harper highlight to exist and return its screen-space center.
 *
 * We return screen coordinates instead of a DOM node because some editors replace parts of
 * the DOM during updates. Coordinates are still usable even if the original highlight element
 * gets disconnected and recreated.
 */
export async function waitForHarperHighlightCenter(
	page: Page,
	timeoutMs = 12000,
): Promise<ScreenPoint | null> {
	const highlight = getHarperHighlights(page).first();

	try {
		await highlight.waitFor({ state: 'visible', timeout: timeoutMs });
	} catch {
		return null;
	}

	const box = await highlight.boundingBox();
	if (box == null || box.width <= 0 || box.height <= 0) {
		return null;
	}

	return {
		x: box.x + box.width / 2,
		y: box.y + box.height / 2,
	};
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
	const center = await waitForHarperHighlightCenter(page);
	if (center == null) return false;

	await page.mouse.click(center.x, center.y);
	return true;
}

/**
 * Open the Harper popup by dispatching the same `pointerdown` event Harper receives from
 * the editor itself.
 *
 * This is intentionally different from clicking the floating highlight. Some editors replace
 * parts of the DOM while the popup is opening, which can briefly disconnect and recreate
 * Harper's popup host. Tests for that behavior need to follow the same event path as a real
 * editor interaction.
 */
export async function openHarperPopupFromEditorPointerDown(
	page: Page,
	editor: Locator,
): Promise<boolean> {
	const center = await waitForHarperHighlightCenter(page);
	if (center == null) {
		return false;
	}

	try {
		await editor.dispatchEvent('pointerdown', {
			bubbles: true,
			composed: true,
			button: 0,
			buttons: 1,
			clientX: center.x,
			clientY: center.y,
			pointerId: 1,
			pointerType: 'mouse',
			screenX: center.x,
			screenY: center.y,
		});
		await page.locator('.harper-container').waitFor({ state: 'visible', timeout: 2000 });
		return true;
	} catch {
		return false;
	}
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

/** Exact-match assertion: `toHaveValue` for form elements, `toHaveText` for contenteditable. */
async function assertEditorText(editor: Locator, text: string) {
	if (await isFormElement(editor)) {
		await expect(editor).toHaveValue(text);
	} else {
		await expect(editor).toHaveText(text);
	}
}

/** Substring assertion: `inputValue` for form elements, `toContainText` for contenteditable. */
async function assertEditorContains(editor: Locator, text: string) {
	if (await isFormElement(editor)) {
		const value = await editor.inputValue();
		expect(value).toContain(text);
	} else {
		await expect(editor).toContainText(text);
	}
}

async function isFormElement(editor: Locator): Promise<boolean> {
	return editor.evaluate((el) => el.tagName === 'TEXTAREA' || el.tagName === 'INPUT');
}

/** Test applying a basic suggestion and verify cursor position after replacement. */
export async function testBasicSuggestion(
	testPageUrl: TestPageUrlProvider,
	getEditor: EditorLocatorProvider,
	setup?: (page: Page, editor: Locator) => Promise<void>,
) {
	test('Can apply basic suggestion.', async ({ page }) => {
		test.slow();
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		const editor = getEditor(page);
		if (setup) {
			await setup(page, editor);
		}
		await replaceEditorContent(editor, 'This is an test');

		const opened = await clickHarperHighlight(page);
		expect(opened).toBe(true);
		await page.getByTitle('Replace with "a"').click();

		await page.waitForTimeout(3000);

		await assertEditorText(editor, 'This is a test');

		// Cursor should be right after "a" (pos 9). ArrowRight×3 + Backspace deletes 'e'.
		await page.keyboard.press('ArrowRight');
		await page.keyboard.press('ArrowRight');
		await page.keyboard.press('ArrowRight');
		await page.keyboard.press('Backspace');
		await assertEditorText(editor, 'This is a tst');

		// Verify typing still works.
		await editor.pressSequentially('e');
		await assertEditorText(editor, 'This is a test');
	});
}

/** Test ignoring a suggestion. */
export async function testCanIgnoreSuggestion(
	testPageUrl: TestPageUrlProvider,
	getEditor: EditorLocatorProvider,
	setup?: (page: Page, editor: Locator) => Promise<void>,
) {
	test('Can ignore suggestion.', async ({ page }) => {
		test.slow();
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		const editor = getEditor(page);
		if (setup) {
			await setup(page, editor);
		}

		const cacheSalt = randomString(5);
		await replaceEditorContent(editor, cacheSalt);

		// Open the popup for the first highlight and click Ignore.
		const opened = await clickHarperHighlight(page);
		expect(opened).toBe(true);
		await page.getByTitle('Ignore this lint').click();

		// Wait for highlights to disappear after ignoring.
		await expect(getHarperHighlights(page)).toHaveCount(0);

		// Nothing should change.
		await assertEditorText(editor, cacheSalt);
		expect(await clickHarperHighlight(page)).toBe(false);
		await assertLocatorIsFocused(page, editor);

		// Backspace at position 0 is a no-op; unchanged text means cursor jumped.
		await page.waitForTimeout(300);
		await page.keyboard.press('Backspace');
		await page.waitForTimeout(300);
		if (await isFormElement(editor)) {
			await expect(editor).not.toHaveValue(cacheSalt);
		} else {
			await expect(editor).not.toHaveText(cacheSalt);
		}
	});
}

/** Test disabling a lint rule via the block button. */
export async function testCanBlockRuleSuggestion(
	testPageUrl: TestPageUrlProvider,
	getEditor: EditorLocatorProvider,
	setup?: (page: Page, editor: Locator) => Promise<void>,
) {
	test('Can hide with rule block button', async ({ page }) => {
		test.slow();
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		const editor = getEditor(page);
		if (setup) {
			await setup(page, editor);
		}
		await replaceEditorContent(editor, 'This is an test.');

		const opened = await clickHarperHighlight(page);
		expect(opened).toBe(true);

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
	setup?: (page: Page, editor: Locator) => Promise<void>,
) {
	test('Multiple suggestions and undo.', async ({ page }) => {
		const url = await resolveTestPage(testPageUrl, page);
		await page.goto(url);

		const editor = getEditor(page);
		if (setup) {
			await setup(page, editor);
		}
		await replaceEditorContent(editor, 'The first tset.\nThe second tset.\nThe third tset.');

		await page.waitForTimeout(6000);

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
		await assertEditorContains(editor, 'first tset');
		await assertEditorContains(editor, 'second test');
		await assertEditorContains(editor, 'third tset');

		// Undo
		await editor.press('Control+z');
		await page.waitForTimeout(300);
		await assertEditorContains(editor, 'The second tset');
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

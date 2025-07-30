import type { Locator, Page } from '@playwright/test';

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

	const box = await highlights.boundingBox();

	if (box == null) {
		return false;
	}

	// Locate the center of the element.
	const cx = box.x + box.width / 2;
	const cy = box.y + box.height / 2;

	await page.mouse.click(cx, cy);
	return true;
}

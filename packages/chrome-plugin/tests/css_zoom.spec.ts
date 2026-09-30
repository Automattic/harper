import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { getHarperHighlights, getTextarea, replaceEditorContent } from './testUtils';

test('Positions highlights inside an ancestor with CSS zoom.', async ({ page }) => {
	await page.goto('http://localhost:8081/css_zoom.html');

	const editor = getTextarea(page);
	await replaceEditorContent(editor, 'This is an test of the Harper grammar checker.');

	const highlights = getHarperHighlights(page);
	await expect(highlights).toHaveCount(1, { timeout: 24000 });

	const editorBox = (await editor.boundingBox())!;
	const highlightBox = (await highlights.first().boundingBox())!;

	// When the zoom is applied to the highlight a second time, it drifts below and to the
	// right of the textarea.
	expect(highlightBox.x).toBeGreaterThanOrEqual(editorBox.x);
	expect(highlightBox.y).toBeGreaterThanOrEqual(editorBox.y);
	expect(highlightBox.x + highlightBox.width).toBeLessThanOrEqual(editorBox.x + editorBox.width);
	expect(highlightBox.y + highlightBox.height).toBeLessThanOrEqual(editorBox.y + editorBox.height);
});

/** The highlight box inside `textareaId`, relative to that textarea's top-left corner. */
async function relativeHighlight(page: Page, textareaId: string) {
	const editor = page.locator(`#${textareaId}`);
	const editorBox = (await editor.boundingBox())!;

	for (const highlight of await getHarperHighlights(page).all()) {
		const box = await highlight.boundingBox();
		if (
			box != null &&
			box.x >= editorBox.x &&
			box.y >= editorBox.y &&
			box.x < editorBox.x + editorBox.width &&
			box.y < editorBox.y + editorBox.height
		) {
			return { editorBox, x: box.x - editorBox.x, y: box.y - editorBox.y, width: box.width };
		}
	}

	return null;
}

test('Positions highlights inside a scaled and zoomed ancestor.', async ({ page }) => {
	await page.goto('http://localhost:8081/css_scale.html');

	// Put the mistake far along the line, so a scale error adds up to a visible offset.
	const text = 'The quick brown fox jumps over the lazy dog, and then an test.';
	await replaceEditorContent(page.locator('#reference'), text);
	await replaceEditorContent(page.locator('#scaled'), text);

	await expect(getHarperHighlights(page)).toHaveCount(2, { timeout: 24000 });

	const reference = (await relativeHighlight(page, 'reference'))!;
	const scaled = (await relativeHighlight(page, 'scaled'))!;
	expect(reference).not.toBeNull();
	expect(scaled).not.toBeNull();

	// Both textareas have the same layout size, so the scaled one should show the same
	// highlight, just scaled by the ratio of their on-screen widths.
	const ratio = scaled.editorBox.width / reference.editorBox.width;
	expect(Math.abs(scaled.x / ratio - reference.x)).toBeLessThan(2);
	expect(Math.abs(scaled.y / ratio - reference.y)).toBeLessThan(2);
	expect(Math.abs(scaled.width / ratio - reference.width)).toBeLessThan(2);
});

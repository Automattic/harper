import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { clickHarperHighlight, getHarperHighlights, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'https://typst.app/play/';

async function assertHighlightCount(page: Page, count: number) {
	await expect(getHarperHighlights(page)).toHaveCount(count, { timeout: 30000 });
}

test('Typst CodeMirror editor can apply a suggestion', async ({ page }) => {
	await page.goto(TEST_PAGE_URL, { waitUntil: 'domcontentloaded' });

	const editor = page.locator('.cm-editor .cm-content[contenteditable="true"]').first();
	await expect(editor).toBeVisible({ timeout: 30000 });
	await replaceEditorContent(editor, 'This is an test');

	await assertHighlightCount(page, 1);

	expect(await clickHarperHighlight(page)).toBe(true);
	await page.getByTitle('Replace with "a"').click();

	await expect(editor).toContainText('This is a test');
});

test('Typst CodeMirror handles multiline suggestions distinctly', async ({ page }) => {
	test.setTimeout(120000);
	await page.goto(TEST_PAGE_URL, { waitUntil: 'domcontentloaded' });

	const editor = page.locator('.cm-editor .cm-content[contenteditable="true"]').first();
	await expect(editor).toBeVisible({ timeout: 30000 });
	await replaceEditorContent(editor, 'First line an test\nSecond line an test');

	await expect(editor).toContainText('First line an test');
	await expect(editor).toContainText('Second line an test');
	await assertHighlightCount(page, 2);
	const initialHighlightCount = 2;
	expect(await clickHarperHighlight(page)).toBe(true);
	await page.getByTitle('Replace with "a"').click();

	const editorText = await editor.innerText();
	expect((editorText.match(/an test/g) ?? []).length).toBe(1);
	expect((editorText.match(/a test/g) ?? []).length).toBe(1);
	await assertHighlightCount(page, initialHighlightCount - 1);
});

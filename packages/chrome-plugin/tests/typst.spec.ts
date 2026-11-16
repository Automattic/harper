import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getHarperHighlights,
	replaceEditorContent,
	waitForHighlightsToSettle,
} from './testUtils';

// The Typst web app uses CodeMirror 6. A local CodeMirror page keeps these tests independent of
// typst.app, whose editor sometimes failed to load on CI.
const TEST_PAGE_URL = 'http://localhost:8081/codemirror.html';

/** Wait for the CDN module and editor before sending keyboard input. */
async function openEditor(page: Page) {
	await page.goto(TEST_PAGE_URL, { waitUntil: 'domcontentloaded' });
	await expect(page.locator('#status')).toHaveText('Ready', { timeout: 30000 });

	const editor = page.locator('.cm-editor .cm-content[contenteditable="true"]').first();
	await expect(editor).toBeVisible();
	return editor;
}

async function assertHighlightCount(page: Page, count: number) {
	await expect(getHarperHighlights(page)).toHaveCount(count, { timeout: 30000 });
}

test('Typst CodeMirror editor can apply a suggestion', async ({ page }) => {
	test.slow();
	const editor = await openEditor(page);
	await replaceEditorContent(editor, 'This is an test');

	await assertHighlightCount(page, 1);
	await waitForHighlightsToSettle(page);

	expect(await clickHarperHighlight(page)).toBe(true);
	await page.getByTitle('Click to replace "an" with "a"').click();

	await expect(editor).toContainText('This is a test');
});

for (const blankLines of [1, 3, 5]) {
	test(`Typst CodeMirror can apply a suggestion after ${blankLines} blank lines`, async ({
		page,
	}) => {
		test.slow();
		const editor = await openEditor(page);
		await replaceEditorContent(
			editor,
			`This is correct.${'\n'.repeat(blankLines + 1)}This is an test`,
		);

		await assertHighlightCount(page, 1);
		await waitForHighlightsToSettle(page);

		expect(await clickHarperHighlight(page)).toBe(true);
		await page.getByTitle('Click to replace "an" with "a"').click();

		await expect(editor).toContainText('This is correct.');
		await expect(editor).toContainText('This is a test');
		await assertHighlightCount(page, 0);
	});
}

test('Typst CodeMirror handles multiline suggestions distinctly', async ({ page }) => {
	test.setTimeout(120000);
	const editor = await openEditor(page);
	await replaceEditorContent(editor, 'First line an test\nSecond line an test');

	await expect(editor).toContainText('First line an test');
	await expect(editor).toContainText('Second line an test');
	await assertHighlightCount(page, 2);
	await waitForHighlightsToSettle(page);
	const initialHighlightCount = 2;
	expect(await clickHarperHighlight(page)).toBe(true);
	await page.getByTitle('Click to replace "an" with "a"').click();

	const editorText = await editor.innerText();
	expect((editorText.match(/an test/g) ?? []).length).toBe(1);
	expect((editorText.match(/a test/g) ?? []).length).toBe(1);
	await assertHighlightCount(page, initialHighlightCount - 1);
});

import { expect, test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	assertLocatorIsFocused,
	clickHarperHighlight,
	getBackground,
	getHarperHighlights,
	getTextarea,
	replaceEditorContent,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/simple_textarea.html';

testBasicSuggestion(TEST_PAGE_URL, getTextarea);
testCanIgnoreSuggestion(TEST_PAGE_URL, getTextarea);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getTextarea);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getTextarea);

test('Wraps correctly', async ({ page }, testInfo) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if \nit is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	if (testInfo.project.name == 'chromium') {
		await assertHarperHighlightBoxes(page, [
			{
				x: 196.52084350585938,
				y: 48.66666793823242,
				width: 39.96875,
				height: 21.322917938232422,
			},
			{
				x: 236.5,
				y: 48.66666793823242,
				width: 6.6666717529296875,
				height: 21.322917938232422,
			},
			{
				x: 10,
				y: 68,
				width: 6.666667938232422,
				height: 21.333335876464844,
			},
			{
				x: 203.17709350585938,
				y: 29.33333396911621,
				width: 20,
				height: 21.33333396911621,
			},
		]);
	} else {
		await assertHarperHighlightBoxes(page, [
			{ x: 178, y: 28, width: 48, height: 20 },
			{ x: 358, y: 10, width: 18, height: 20 },
		]);
	}
});

test('Scrolls correctly', async ({ page }) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the the Harper grammar checker, specifically if \n\n\n\n\n\n\n\n\n\n\n\n\nit scrolls beyo nd the height of the buffer.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [{ height: 19, width: 56, x: 97.953125, y: 63 }]);
});

test.describe('textarea lint delay', () => {
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('Remaps textarea highlights during lint delay when typing before a lint', async ({
		page,
		context,
	}) => {
		const background = await getBackground(context);
		await background.evaluate(async () => {
			await chrome.storage.local.set({ delay: 10000 });
		});

		await page.goto(TEST_PAGE_URL);

		const editor = getTextarea(page);
		await replaceEditorContent(editor, 'This is an test');

		const highlight = getHarperHighlights(page).first();
		await highlight.waitFor({ state: 'visible' });
		const before = await highlight.boundingBox();
		expect(before).not.toBeNull();

		await editor.evaluate((el: HTMLTextAreaElement) => {
			el.setSelectionRange('This is '.length, 'This is '.length);
			el.focus();
		});
		await editor.pressSequentially('really ');

		await expect
			.poll(async () => (await highlight.boundingBox())?.x ?? null, { timeout: 2000 })
			.toBeGreaterThan((before?.x ?? 0) + 35);
	});

	test('Hides stale textarea highlights during lint delay when editing inside a lint', async ({
		page,
		context,
	}) => {
		const background = await getBackground(context);
		await background.evaluate(async () => {
			await chrome.storage.local.set({ delay: 10000 });
		});

		await page.goto(TEST_PAGE_URL);

		const editor = getTextarea(page);
		await replaceEditorContent(editor, 'This is an test');

		const highlights = getHarperHighlights(page);
		await highlights.first().waitFor({ state: 'visible' });

		await editor.evaluate((el: HTMLTextAreaElement) => {
			el.setSelectionRange('This is a'.length, 'This is a'.length);
			el.focus();
		});
		await editor.pressSequentially('x');

		await expect.poll(async () => await highlights.count(), { timeout: 2000 }).toBe(0);
	});
});

test('Can dismiss with escape key', async ({ page }) => {
	test.slow();
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if it is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	await clickHarperHighlight(page);

	await page.locator('.harper-container').waitFor({ state: 'visible' });

	await page.keyboard.press('Escape');

	await page.locator('.harper-container').waitFor({ state: 'hidden' });

	await assertLocatorIsFocused(page, editor);
});

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
			{ x: 233.90625, y: 44, width: 48, height: 19 },
			{ x: 281.90625, y: 44, width: 8, height: 19 },
			{ x: 10, y: 61, width: 8, height: 19 },
			{ x: 241.90625, y: 27, width: 24, height: 19 },
		]);
	} else {
		await assertHarperHighlightBoxes(page, [
			{ x: 10, y: 71, width: 57.599998474121094, height: 17 },
			{ x: 218.8000030517578, y: 26, width: 21.600006103515625, height: 17 },
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
	test.setTimeout(120000); // Increased from default (30000ms) to 120000ms (2 minutes)
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is a test of the Harper grammar checker, specifically   if it is wrapped around a line weirdl y',
	);

	await page.waitForTimeout(6000);

	const startTime = Date.now();
	console.log(`[TIMING] Starting click Harper highlight at ${startTime}`);
	
	await clickHarperHighlight(page);
	
	const clickTime = Date.now();
	console.log(`[TIMING] Harper highlight clicked after ${clickTime - startTime}ms`);

	console.log(`[TIMING] Starting wait for popup container at ${clickTime}`);
	await page.locator('.harper-container').waitFor({ state: 'visible' });
	
	const popupTime = Date.now();
	console.log(`[TIMING] Popup container appeared after ${popupTime - clickTime}ms`);

	await page.keyboard.press('Escape');

	console.log(`[TIMING] Starting wait for popup container to hide at ${popupTime}`);
	await page.locator('.harper-container').waitFor({ state: 'hidden' });
	
	const hideTime = Date.now();
	console.log(`[TIMING] Popup container hidden after ${hideTime - popupTime}ms`);

	await assertLocatorIsFocused(page, editor);
	
	const endTime = Date.now();
	console.log(`[TIMING] Total test duration: ${endTime - startTime}ms`);
});

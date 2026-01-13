import { test } from './fixtures';
import {
	assertHarperHighlightBoxes,
	getTextarea,
	replaceEditorContent,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/nested_elements.html';

test('Positions properly in oddly nested page.', async ({ page }, testInfo) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is an test of the Harper grammar checker, specifically   if \n the highlights are positionasd properly.',
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

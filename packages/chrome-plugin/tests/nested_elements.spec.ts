import { test } from './fixtures';
import { assertHarperHighlightBoxes, getTextarea, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/nested_elements.html';

test('Positions properly in oddly nested page.', async ({ page }, testInfo) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is an test of the Harper grammar checker, specifically   if \n the highlights are positionasd properly.',
	);

	await page.waitForTimeout(6000);

	await assertHarperHighlightBoxes(page, [
		{ x: 396.390625, y: 243, width: 15.625, height: 19 },
		{ x: 794.1875, y: 243, width: 23.421875, height: 19 },
		{ x: 490, y: 260, width: 85.828125, height: 19 },
	]);
});

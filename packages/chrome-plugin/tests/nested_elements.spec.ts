import { test } from './fixtures';
import { assertHarperHighlightBoxes, getTextarea, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/nested_elements.html';

test('Positions properly in oddly nested page.', async ({ page }, _testInfo) => {
	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(
		editor,
		'This is an test of the Harper grammar checker, specifically   if \n the highlights are positionasd properly.',
	);

	await page.waitForTimeout(12000);

	await assertHarperHighlightBoxes(page, [
		[
			{ x: 396.390625, y: 243, width: 15.625, height: 19 },
			{ x: 794.1875, y: 243, width: 23.421875, height: 19 },
			{ x: 490, y: 260, width: 85.828125, height: 19 },
		],
		[
			{ x: 385.3333435058594, y: 242, width: 13, height: 21.333343505859375 },
			{ x: 716.8333129882812, y: 242, width: 19.5, height: 21.333343505859375 },
			{
				x: 463.3333435058594,
				y: 261.3333435058594,
				width: 71.49996948242188,
				height: 21.333343505859375,
			},
		],
	]);
});

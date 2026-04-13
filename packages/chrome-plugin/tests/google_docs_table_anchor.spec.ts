import { expect, test } from './fixtures';
import {
	appendGoogleDocsTable,
	openLiveGoogleDoc,
	replaceGoogleDocsDocumentText,
} from './googleDocsTestUtils';
import { waitForHarperHighlightCenter } from './testUtils';

test.describe('Google Docs support', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(180000);
	test.skip(
		({ browserName }) => browserName !== 'chromium',
		'Live Google Docs runs only on Chromium',
	);

	test.beforeEach(async ({ page }) => {
		await openLiveGoogleDoc(page);
	});

	test.afterEach(async ({ page }) => {
		try {
			await replaceGoogleDocsDocumentText(page, '');
		} catch {
			// Best effort cleanup for the shared document.
		}
	});

	test('Google Docs anchors table-cell typo highlights to the right column text', async ({
		page,
	}) => {
		const suffix = Date.now().toString(36);
		const leftCell = `Dana Mills ${suffix}`;
		const cleanRightCell = 'writes careful notes for the team.';
		const secondLeftCell = `Emily Stone ${suffix}`;
		const typoRightCell = 'This is teh plan.';

		await appendGoogleDocsTable(page, [
			[`Name ${suffix}`, `Notes ${suffix}`],
			[leftCell, cleanRightCell],
			[secondLeftCell, typoRightCell],
		]);

		const highlightCenter = await waitForHarperHighlightCenter(page, 20000);
		expect(highlightCenter).not.toBeNull();

		const tableRects = await page.evaluate(
			({ leftCell, secondLeftCell }) => {
				return Array.from(
					document.querySelectorAll<SVGRectElement>('.kix-appview-editor rect[aria-label]'),
				)
					.map((rect) => {
						const box = rect.getBoundingClientRect();
						return {
							label: rect.getAttribute('aria-label') ?? '',
							x: box.x,
							y: box.y,
							width: box.width,
							height: box.height,
						};
					})
					.filter((rect) => rect.label.includes(leftCell) || rect.label.includes(secondLeftCell));
			},
			{ leftCell, secondLeftCell },
		);

		expect(tableRects.length).toBe(2);
		const leftBoundary = Math.max(...tableRects.map((rect) => rect.x + rect.width));
		expect(highlightCenter!.x).toBeGreaterThan(leftBoundary + 80);
		expect(highlightCenter!.y).toBeGreaterThan(Math.min(...tableRects.map((rect) => rect.y)));
	});
});

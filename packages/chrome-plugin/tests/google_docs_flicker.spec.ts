import { expect, test } from './fixtures';
import {
	appendGoogleDocsText,
	getGoogleDocsHighlightCount,
	openLiveGoogleDoc,
	replaceGoogleDocsDocumentText,
} from './googleDocsTestUtils';

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

	test('Google Docs keeps existing typo highlights stable while unrelated text is appended (#3122)', async ({
		page,
	}) => {
		const initialText = 'Stable context. This is teh plan.';
		await replaceGoogleDocsDocumentText(page, initialText);

		await expect
			.poll(() => getGoogleDocsHighlightCount(page), { timeout: 20000 })
			.toBeGreaterThan(0);

		await appendGoogleDocsText(page, ' More text.');

		await expect
			.poll(() => getGoogleDocsHighlightCount(page), { timeout: 20000 })
			.toBeGreaterThan(0);
	});
});

import { expect, test } from './fixtures';
import {
	appendGoogleDocsTable,
	applyBulletedListToGoogleDocsText,
	applyItalicToGoogleDocsText,
	getGoogleDocsAnnotatedText,
	getGoogleDocsBridgeText,
	getGoogleDocsHighlightCount,
	getGoogleDocsNormalizedBridgeText,
	getGoogleDocsVisualLineCount,
	hasGoogleDocsRectWithFontContainingText,
	openLiveGoogleDoc,
	replaceGoogleDocsDocumentText,
	setGoogleDocsFontSize,
} from './googleDocsTestUtils';

const WRAPPED_SENTENCE =
	'This intentionally long sentence wraps across the page so Harper must treat it as one continuous sentence even when Google Docs renders it over multiple visual lines for the reader.';

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

	test('Google Docs restores spaces around formatted inline words', async ({ page }) => {
		await replaceGoogleDocsDocumentText(page, 'not smart enough.');
		await applyItalicToGoogleDocsText(page, 'smart');

		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('not smart enough.');
		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).not.toContain('\n');
	});

	test('Google Docs formatted rects stay in the same sentence', async ({ page }) => {
		await replaceGoogleDocsDocumentText(page, 'This is an test.');
		await applyItalicToGoogleDocsText(page, 'test');

		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('This is an test.');
		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).not.toContain('\n');
	});

	test('Google Docs does not invent spaces around standalone punctuation rects', async ({
		page,
	}) => {
		await replaceGoogleDocsDocumentText(page, 'not smart enough. But');
		await applyItalicToGoogleDocsText(page, 'smart');

		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('not smart enough. But');
		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).not.toContain(' .');
	});

	test('Google Docs inline formatting keeps visible rect styling intact', async ({ page }) => {
		await replaceGoogleDocsDocumentText(page, 'Testing formatting matters.');
		await applyItalicToGoogleDocsText(page, 'formatting');

		await expect
			.poll(() => hasGoogleDocsRectWithFontContainingText(page, 'formatting', /italic/i), {
				timeout: 15000,
			})
			.toBe(true);
		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('Testing formatting matters.');
	});

	test('Google Docs keeps soft wraps out of the logical bridge text', async ({ page }) => {
		await setGoogleDocsFontSize(page, 18);
		await replaceGoogleDocsDocumentText(page, WRAPPED_SENTENCE);

		await expect
			.poll(() => getGoogleDocsVisualLineCount(page), { timeout: 20000 })
			.toBeGreaterThan(1);
		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe(WRAPPED_SENTENCE);
	});

	test('Google Docs preserves paragraph breaks from annotated text', async ({ page }) => {
		await replaceGoogleDocsDocumentText(page, 'First paragraph.\n\nSecond paragraph.');

		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe('First paragraph.\n\nSecond paragraph.');
	});

	test('Google Docs keeps numbered list markers with closing parentheses on single list lines', async ({
		page,
	}) => {
		const text =
			'This paragraph stays sentence case.\n1) This list item should stay sentence case\n2) Another list item should stay sentence case';
		await replaceGoogleDocsDocumentText(page, text);

		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).toBe(text);
	});

	test('Google Docs keeps NBSP-separated formatted inline words in the same sentence', async ({
		page,
	}) => {
		await replaceGoogleDocsDocumentText(page, 'not\u00a0smart enough.');
		await applyItalicToGoogleDocsText(page, 'smart');

		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('not smart enough.');
	});

	test('Google Docs does not misread bullet lists as headings', async ({ page }) => {
		await replaceGoogleDocsDocumentText(
			page,
			'This paragraph stays sentence case.\nThis list item should stay sentence case\nAnother list item should stay sentence case',
		);
		await applyBulletedListToGoogleDocsText(page, 'This list item should stay sentence case');

		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe(
				'This paragraph stays sentence case.\n● This list item should stay sentence case\n● Another list item should stay sentence case',
			);
		await expect.poll(() => getGoogleDocsHighlightCount(page), { timeout: 15000 }).toBe(0);
	});

	test('Google Docs keeps table text in logical row order', async ({ page }) => {
		const suffix = Date.now().toString(36);
		const leftCell = `Leah Pring ${suffix}`;
		const rightCell = 'shares planning notes with the team every morning.';
		const secondLeftCell = `Emily Puetz ${suffix}`;
		const secondRightCell = 'is available in the early morning before 9 and after 3:30.';

		await appendGoogleDocsTable(page, [
			[`Name ${suffix}`, `Notes ${suffix}`],
			[leftCell, rightCell],
			[secondLeftCell, secondRightCell],
		]);

		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).toContain(leftCell);
		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).toContain(rightCell);
		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toContain(secondLeftCell);
		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toContain(secondRightCell);
	});
});

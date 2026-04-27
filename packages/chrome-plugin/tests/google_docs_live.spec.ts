import type { Page } from '@playwright/test';
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
import { clickHarperHighlight, waitForHarperHighlightCenter } from './testUtils';

const WRAPPED_SENTENCE =
	'This intentionally long sentence wraps across the page so Harper must treat it as one continuous sentence even when Google Docs renders it over multiple visual lines for the reader.';
const WRAPPED_LINKED_SENTENCE =
	'We will be clear with one another about exactly what work we expect to finish and by when. For this, we will be tracking project work using the Basecamp tool (https://basecamp.com/; our team page is https://3.basecamp.com/4507520). When timelines can not be met as originally promised, the owner of the task agrees to communicate that change as quickly as possible to impacted stakeholders. Leah Pring will coordinate usage of the Basecamp tool across all workstreams. Invitations will be sent to everyone on the team.';
const GOOGLE_DOCS_BRIDGE_PROTOCOL = 'harper-gdocs-bridge/v1';
const GOOGLE_DOCS_FONT_SIZES = [11, 14, 18];
const LIVE_GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS = 20000;

type ScreenRect = {
	x: number;
	y: number;
	width: number;
	height: number;
};

function getOccurrenceRange(source: string, needle: string, occurrence = 0) {
	let start = -1;
	let fromIndex = 0;

	for (let i = 0; i <= occurrence; i += 1) {
		start = source.indexOf(needle, fromIndex);
		if (start < 0) {
			throw new Error(`Could not find occurrence ${occurrence} of "${needle}"`);
		}
		fromIndex = start + needle.length;
	}

	return {
		start,
		end: start + needle.length,
	};
}

async function dispatchGoogleDocsBridgeReplacement(
	page: Page,
	request: {
		start: number;
		end: number;
		replacementText: string;
		expectedText: string;
		beforeContext: string;
		afterContext: string;
	},
): Promise<boolean> {
	return await page.evaluate(
		async ({ protocol, request }) => {
			const requestId = `playwright-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;

			const response = await new Promise<{ kind: string; applied?: boolean; message?: string }>(
				(resolve, reject) => {
					const timeoutId = window.setTimeout(() => {
						document.removeEventListener('harper:gdocs:response', onResponse as EventListener);
						reject(new Error('Timed out waiting for Google Docs bridge response'));
					}, 5000);

					const onResponse = (event: Event) => {
						const detail = (event as CustomEvent).detail;
						if (detail?.protocol !== protocol || detail?.requestId !== requestId) {
							return;
						}

						window.clearTimeout(timeoutId);
						document.removeEventListener('harper:gdocs:response', onResponse as EventListener);
						resolve(detail.response);
					};

					document.addEventListener('harper:gdocs:response', onResponse as EventListener);
					document.dispatchEvent(
						new CustomEvent('harper:gdocs:request', {
							detail: {
								protocol,
								requestId,
								request: {
									kind: 'replaceText',
									...request,
								},
							},
						}),
					);
				},
			);

			if (response.kind !== 'replaceText') {
				throw new Error(response.message ?? 'Unexpected Google Docs bridge response');
			}

			return response.applied === true;
		},
		{
			protocol: GOOGLE_DOCS_BRIDGE_PROTOCOL,
			request,
		},
	);
}

async function getHarperHighlightBoxes(page: Page): Promise<ScreenRect[]> {
	return await page.locator('#harper-highlight').evaluateAll((nodes) =>
		nodes.map((node) => {
			const rect = node.getBoundingClientRect();
			return {
				x: rect.x,
				y: rect.y,
				width: rect.width,
				height: rect.height,
			};
		}),
	);
}

test.describe('Google Docs live regressions', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(180000);

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

	test('detects basic lints in a live Google Doc (#1966)', async ({ page }) => {
		test.slow();

		await replaceGoogleDocsDocumentText(page, 'This is an test');

		await expect
			.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 })
			.toBe('This is an test');
		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('This is an test');
		expect(await waitForHarperHighlightCenter(page, 20000)).not.toBeNull();
	});

	test('highlights only the incorrect portion in a live Google Doc', async ({ page }) => {
		test.slow();

		const source = 'This is an test';
		const typoText = 'an';
		await replaceGoogleDocsDocumentText(page, source);

		await expect.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 }).toBe(source);
		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe(source);
		expect(
			await waitForHarperHighlightCenter(page, LIVE_GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS),
		).not.toBeNull();

		const highlightBoxes = await getHarperHighlightBoxes(page);
		expect(highlightBoxes.length).toBeGreaterThan(0);
		const sourceRects = await page.evaluate((lineText) => {
			return Array.from(
				document.querySelectorAll<SVGRectElement>('.kix-appview-editor rect[aria-label]'),
			)
				.map((rect) => {
					const label = rect.getAttribute('aria-label') ?? '';
					const box = rect.getBoundingClientRect();
					return {
						label,
						x: box.x,
						y: box.y,
						width: box.width,
						height: box.height,
					};
				})
				.filter((rect) => rect.label.includes(lineText));
		}, source);
		expect(sourceRects.length).toBeGreaterThan(0);

		const sourceRect = sourceRects[0];
		const matchingHighlight = highlightBoxes.find((box) => {
			const centerY = box.y + box.height / 2;
			return centerY >= sourceRect.y - 8 && centerY <= sourceRect.y + sourceRect.height + 8;
		});

		expect(matchingHighlight).toBeDefined();
		expect(matchingHighlight!.width).toBeLessThan(sourceRect.width / 2);
		expect(matchingHighlight!.x).toBeGreaterThan(sourceRect.x + sourceRect.width * 0.2);
		expect(matchingHighlight!.x + matchingHighlight!.width).toBeLessThan(
			sourceRect.x + sourceRect.width * 0.8,
		);
	});

	test('applies suggestion replacements in a live Google Doc (#2995)', async ({ page }) => {
		test.slow();

		await replaceGoogleDocsDocumentText(page, 'This is an test');

		const opened = await clickHarperHighlight(page);
		expect(opened).toBe(true);
		await page.getByTitle('Replace with "a"').click();

		await expect
			.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 })
			.toBe('This is a test');
		await expect
			.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
			.toBe('This is a test');
	});

	test('keeps inline formatting in the same sentence (#2882)', async ({ page }) => {
		test.slow();

		await replaceGoogleDocsDocumentText(page, 'This is not smart enough.');
		await applyItalicToGoogleDocsText(page, 'smart');

		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe('This is not smart enough.');
		await expect.poll(() => getGoogleDocsHighlightCount(page), { timeout: 15000 }).toBe(0);
	});

	test('preserves inline formatting after applying a nearby suggestion', async ({ page }) => {
		test.slow();

		await replaceGoogleDocsDocumentText(page, 'This is an test.');
		await applyItalicToGoogleDocsText(page, 'test');

		await expect
			.poll(() => hasGoogleDocsRectWithFontContainingText(page, 'test', /italic/i), {
				timeout: 15000,
			})
			.toBe(true);

		const opened = await clickHarperHighlight(page);
		expect(opened).toBe(true);
		await page.getByTitle('Replace with "a"').click();

		await expect
			.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 })
			.toBe('This is a test.');
		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe('This is a test.');
		await expect
			.poll(() => hasGoogleDocsRectWithFontContainingText(page, 'test', /italic/i), {
				timeout: 15000,
			})
			.toBe(true);
	});

	test('does not misread bullet lists as headings (#2923)', async ({ page }) => {
		test.slow();

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

	test('preserves paragraph boundaries in the bridge text (#2959)', async ({ page }) => {
		test.slow();

		await replaceGoogleDocsDocumentText(
			page,
			'First short paragraph\nSecond short paragraph\nThird short paragraph',
		);

		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe('First short paragraph\n\nSecond short paragraph\n\nThird short paragraph');
	});

	test('realigns stale replacement offsets to the intended duplicate phrase', async ({ page }) => {
		test.slow();

		const originalText = 'Context alpha: This is an test. Context beta: This is an test.';
		const insertedPrefix = 'Long inserted prefix here. ';
		await replaceGoogleDocsDocumentText(page, originalText);
		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).toBe(originalText);

		const originalBridgeText = await getGoogleDocsBridgeText(page);
		const targetSpan = getOccurrenceRange(originalBridgeText, 'an', 1);
		const beforeContext = originalBridgeText.slice(
			Math.max(0, targetSpan.start - 64),
			targetSpan.start,
		);
		const afterContext = originalBridgeText.slice(
			targetSpan.end,
			Math.min(originalBridgeText.length, targetSpan.end + 64),
		);

		const shiftedText = `Context alpha: This is an test. Context beta: ${insertedPrefix}This is an test.`;
		await replaceGoogleDocsDocumentText(page, shiftedText);

		const applied = await dispatchGoogleDocsBridgeReplacement(page, {
			start: targetSpan.start,
			end: targetSpan.end,
			replacementText: 'a',
			expectedText: originalBridgeText.slice(targetSpan.start, targetSpan.end),
			beforeContext,
			afterContext,
		});

		expect(applied).toBe(true);
		await expect
			.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 })
			.toBe(
				'Context alpha: This is an test. Context beta: Long inserted prefix here. This is a test.',
			);
		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe(
				'Context alpha: This is an test. Context beta: Long inserted prefix here. This is a test.',
			);
	});

	test('realigns stale replacement offsets when duplicate local contexts are ambiguous', async ({
		page,
	}) => {
		test.slow();

		const repeatedSentence = 'Lorem ipsum an dolor sit amet.';
		const originalText = `${repeatedSentence} ${repeatedSentence}`;
		const insertedPrefix =
			'Long inserted prefix that shifts the target far away from its original offsets. ';

		await replaceGoogleDocsDocumentText(page, originalText);
		await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 }).toBe(originalText);

		const originalBridgeText = await getGoogleDocsBridgeText(page);
		const targetSpan = getOccurrenceRange(originalBridgeText, 'an', 1);
		const beforeContext = originalBridgeText.slice(
			Math.max(0, targetSpan.start - 64),
			targetSpan.start,
		);
		const afterContext = originalBridgeText.slice(
			targetSpan.end,
			Math.min(originalBridgeText.length, targetSpan.end + 64),
		);

		const shiftedText = `${repeatedSentence} ${insertedPrefix}${repeatedSentence}`;
		await replaceGoogleDocsDocumentText(page, shiftedText);

		const applied = await dispatchGoogleDocsBridgeReplacement(page, {
			start: targetSpan.start,
			end: targetSpan.end,
			replacementText: 'a',
			expectedText: originalBridgeText.slice(targetSpan.start, targetSpan.end),
			beforeContext,
			afterContext,
		});

		expect(applied).toBe(true);
		await expect
			.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 })
			.toBe(`Lorem ipsum an dolor sit amet. ${insertedPrefix}Lorem ipsum a dolor sit amet.`);
		await expect
			.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
			.toBe(`Lorem ipsum an dolor sit amet. ${insertedPrefix}Lorem ipsum a dolor sit amet.`);
	});

	for (const fontSize of GOOGLE_DOCS_FONT_SIZES) {
		test(`does not invent a sentence break at a soft wrap (#2992, ${fontSize}pt)`, async ({
			page,
		}) => {
			test.slow();

			await setGoogleDocsFontSize(page, fontSize);
			await replaceGoogleDocsDocumentText(page, WRAPPED_SENTENCE);

			await expect
				.poll(() => getGoogleDocsVisualLineCount(page), { timeout: 20000 })
				.toBeGreaterThan(1);
			await expect
				.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 20000 })
				.toBe(WRAPPED_SENTENCE);
			await expect
				.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
				.toBe(WRAPPED_SENTENCE);
		});

		test(`keeps wrapped url-heavy prose intact on a real Google Doc page (${fontSize}pt)`, async ({
			page,
		}) => {
			test.slow();

			await setGoogleDocsFontSize(page, fontSize);
			await replaceGoogleDocsDocumentText(page, WRAPPED_LINKED_SENTENCE);

			await expect
				.poll(() => getGoogleDocsVisualLineCount(page), { timeout: 20000 })
				.toBeGreaterThan(3);
			await expect
				.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
				.toBe(WRAPPED_LINKED_SENTENCE);
			await expect.poll(() => getGoogleDocsHighlightCount(page), { timeout: 15000 }).toBe(0);
		});

		test(`does not collapse early morning inside a real Google Docs table cell (${fontSize}pt)`, async ({
			page,
		}) => {
			test.slow();

			const suffix = `${fontSize}-${Date.now().toString(36)}`;
			const leftCell = `Emily Puetz ${suffix}`;
			const rightCell =
				'Is available in the early morning before 9 and after 3:30, then reviews drafts again after lunch on most weekdays.';

			await setGoogleDocsFontSize(page, fontSize);
			await appendGoogleDocsTable(page, [
				[`Name ${suffix}`, `Availability notes ${suffix}`],
				[leftCell, rightCell],
			]);

			await expect
				.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 })
				.toContain(rightCell);
			await expect
				.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
				.toContain('early morning before 9 and after 3:30,');
			await expect
				.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
				.toContain('then reviews drafts again after lunch on most weekdays.');
			await expect
				.poll(() => getGoogleDocsBridgeText(page), { timeout: 20000 })
				.not.toContain('earlymorning');
			const getTableRects = async () =>
				await page.evaluate(
					({ leftCell }) => {
						const rects = Array.from(
							document.querySelectorAll<SVGRectElement>('.kix-appview-editor rect[aria-label]'),
						).map((rect) => {
							const box = rect.getBoundingClientRect();
							return {
								label: rect.getAttribute('aria-label') ?? '',
								top: Math.round(box.top),
								left: Math.round(box.left),
							};
						});

						const leftRect = rects.find((rect) => rect.label.includes(leftCell));
						const rightRects =
							leftRect == null
								? []
								: rects.filter(
										(rect) => rect.left > leftRect.left + 100 && rect.top >= leftRect.top - 12,
									);
						if (!leftRect || rightRects.length < 2) {
							return null;
						}

						return {
							leftTop: leftRect.top,
							leftX: leftRect.left,
							rightRects,
						};
					},
					{
						leftCell: 'Emily Puetz',
					},
				);
			await expect.poll(() => getTableRects(), { timeout: 20000 }).not.toBeNull();
			const tableRects = await getTableRects();
			const highestRightRect = Math.min(...tableRects!.rightRects.map(({ top }) => top));
			const leftmostRightRect = Math.min(...tableRects!.rightRects.map(({ left }) => left));
			expect(Math.abs(tableRects!.leftTop - highestRightRect)).toBeLessThanOrEqual(12);
			expect(leftmostRightRect - tableRects!.leftX).toBeGreaterThan(100);
			expect(new Set(tableRects!.rightRects.map(({ top }) => top)).size).toBeGreaterThan(1);

			const highlightCountInTargetRow = await page.evaluate(
				({ leftBoundary, rowTop, rowBottom }) => {
					return Array.from(document.querySelectorAll<HTMLElement>('#harper-highlight')).filter(
						(highlight) => {
							const box = highlight.getBoundingClientRect();
							const centerX = box.left + box.width / 2;
							const centerY = box.top + box.height / 2;
							return centerX > leftBoundary && centerY >= rowTop && centerY <= rowBottom;
						},
					).length;
				},
				{
					leftBoundary: tableRects!.leftX + 100,
					rowTop: highestRightRect - 12,
					rowBottom: Math.max(...tableRects!.rightRects.map(({ top }) => top)) + 24,
				},
			);
			expect(highlightCountInTargetRow).toBe(0);
		});
	}
});

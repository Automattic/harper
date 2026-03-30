import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';

type MockGoogleDocsRect = {
	label: string;
	left: number;
	top: number;
	width: number;
	height: number;
	fontCss?: string;
};

const TEST_PAGE_URL = 'https://docs.google.com/document/d/harper-formatting-regression/edit';
const FORMATTED_WORD_GAP_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'not',
		left: 48,
		top: 48,
		width: 24,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'smart',
		left: 80,
		top: 56,
		width: 42,
		height: 18,
		fontCss: 'italic 16px Arial',
	},
	{
		label: 'enough.',
		left: 130,
		top: 48,
		width: 60,
		height: 18,
		fontCss: '16px Arial',
	},
];
const ITALIC_SHIFTED_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'This is an ',
		left: 48,
		top: 48,
		width: 96,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'test.',
		left: 144,
		top: 56,
		width: 48,
		height: 18,
		fontCss: 'italic 16px Arial',
	},
];
const PUNCTUATION_BOUNDARY_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'not',
		left: 48,
		top: 48,
		width: 24,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'smart',
		left: 80,
		top: 56,
		width: 42,
		height: 18,
		fontCss: 'italic 16px Arial',
	},
	{
		label: 'enough',
		left: 130,
		top: 48,
		width: 54,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: '.',
		left: 184,
		top: 48,
		width: 6,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'But',
		left: 202,
		top: 48,
		width: 26,
		height: 18,
		fontCss: '16px Arial',
	},
];
const SUPERSCRIPT_LIKE_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'Testing ',
		left: 48,
		top: 96,
		width: 68,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'formatting',
		left: 116,
		top: 88,
		width: 72,
		height: 11,
		fontCss: 'bold 11px Arial',
	},
	{
		label: ' matters.',
		left: 188,
		top: 96,
		width: 64,
		height: 18,
		fontCss: '16px Arial',
	},
];
const WRAPPED_LINE_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'This sentence wraps',
		left: 48,
		top: 48,
		width: 128,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'because the viewport is narrow.',
		left: 48,
		top: 72,
		width: 182,
		height: 18,
		fontCss: '16px Arial',
	},
];
const PARAGRAPH_BREAK_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'First paragraph.',
		left: 48,
		top: 48,
		width: 110,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'Second paragraph.',
		left: 48,
		top: 96,
		width: 126,
		height: 18,
		fontCss: '16px Arial',
	},
];
const NUMBERED_PAREN_LIST_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'This paragraph stays sentence case.',
		left: 48,
		top: 48,
		width: 220,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: '1)',
		left: 48,
		top: 96,
		width: 18,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'This list item should stay sentence case',
		left: 74,
		top: 96,
		width: 240,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: '2)',
		left: 48,
		top: 120,
		width: 18,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'Another list item should stay sentence case',
		left: 74,
		top: 120,
		width: 258,
		height: 18,
		fontCss: '16px Arial',
	},
];
const NBSP_FORMATTED_GAP_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'not',
		left: 48,
		top: 48,
		width: 24,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'smart',
		left: 74,
		top: 56,
		width: 42,
		height: 18,
		fontCss: 'italic 16px Arial',
	},
	{
		label: 'enough.',
		left: 122,
		top: 48,
		width: 60,
		height: 18,
		fontCss: '16px Arial',
	},
];
const EM_DASH_SPLIT_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'word',
		left: 48,
		top: 48,
		width: 34,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: '—',
		left: 86,
		top: 48,
		width: 10,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'word',
		left: 100,
		top: 48,
		width: 34,
		height: 18,
		fontCss: '16px Arial',
	},
];
const WIDE_SAME_ROW_GAP_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'Name',
		left: 48,
		top: 48,
		width: 38,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'Value',
		left: 176,
		top: 48,
		width: 40,
		height: 18,
		fontCss: '16px Arial',
	},
];
const COLUMN_MAJOR_TABLE_RECTS: MockGoogleDocsRect[] = [
	{
		label: 'Leah Pring',
		left: 48,
		top: 48,
		width: 92,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'Emily Puetz',
		left: 48,
		top: 120,
		width: 96,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'shares planning notes with the team every',
		left: 296,
		top: 48,
		width: 300,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'morning.',
		left: 296,
		top: 72,
		width: 64,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'is available in the early',
		left: 296,
		top: 120,
		width: 176,
		height: 18,
		fontCss: '16px Arial',
	},
	{
		label: 'morning before 9 and after 3:30.',
		left: 296,
		top: 144,
		width: 236,
		height: 18,
		fontCss: '16px Arial',
	},
];
const COLUMN_MAJOR_TABLE_TEXT = [
	'Leah Pring',
	'shares planning notes with the team every morning.',
	'Emily Puetz',
	'is available in the early morning before 9 and after 3:30.',
].join('\n');
const COLUMN_MAJOR_TABLE_COLLAPSED_WORD_TEXT = [
	'Leah Pring',
	'shares planning notes with the team every morning.',
	'Emily Puetz',
	'is available in the earlymorning before 9 and after 3:30.',
].join('\n');
function buildMockGoogleDocsHtml(rects: MockGoogleDocsRect[]): string {
	const rectMarkup = rects
		.map(
			(rect, index) => `
				<rect
					data-rect-index="${index}"
					x="${rect.left}"
					y="${rect.top}"
					width="${rect.width}"
					height="${rect.height}"
					fill="rgba(15, 23, 42, 0.001)"
					aria-label="${escapeHtml(rect.label)}"
					data-font-css="${escapeHtml(rect.fontCss ?? '')}"
				/>
			`,
		)
		.join('');

	return `<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<title>Mock Google Docs</title>
		<style>
			body {
				margin: 0;
				font: 16px/1.4 sans-serif;
				background: #f3f4f6;
			}

			#docs-editor {
				position: relative;
				width: 900px;
				height: 240px;
				margin: 32px auto;
				background: white;
				border: 1px solid #d1d5db;
			}

			svg {
				width: 100%;
				height: 100%;
			}
		</style>
	</head>
	<body>
		<div id="docs-editor" class="kix-appview-editor">
			<svg>${rectMarkup}</svg>
		</div>
	</body>
</html>`;
}

function escapeHtml(text: string): string {
	return text
		.replaceAll('&', '&amp;')
		.replaceAll('"', '&quot;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;');
}

async function mockGoogleDocsPage(page: Page, rects: MockGoogleDocsRect[], url = TEST_PAGE_URL) {
	await page.route(url, async (route) => {
		await route.fulfill({
			contentType: 'text/html',
			body: buildMockGoogleDocsHtml(rects),
		});
	});
}

async function installMockGoogleDocsGeometry(
	page: Page,
	rects: MockGoogleDocsRect[],
	annotatedText = rects.map((rect) => rect.label).join(''),
) {
	await page.evaluate(
		({ pageText }) => {
			(
				window as Window & {
					__harperMockGoogleDocsText?: string;
					_docs_annotate_getAnnotatedText?: () => Promise<{
						getText: () => string;
						setSelection: () => void;
						getSelection: () => Array<{ start: number; end: number }>;
					}>;
				}
			).__harperMockGoogleDocsText = pageText;
			(
				window as Window & {
					__harperMockGoogleDocsText?: string;
					_docs_annotate_getAnnotatedText?: () => Promise<{
						getText: () => string;
						setSelection: () => void;
						getSelection: () => Array<{ start: number; end: number }>;
					}>;
				}
			)._docs_annotate_getAnnotatedText = async () => ({
				getText: () =>
					(
						window as Window & {
							__harperMockGoogleDocsText?: string;
						}
					).__harperMockGoogleDocsText ?? '',
				setSelection: () => {},
				getSelection: () => [{ start: 0, end: 0 }],
			});
		},
		{ pageText: annotatedText },
	);
}


async function openMockGoogleDocsPage(
	page: Page,
	rects: MockGoogleDocsRect[],
	annotatedText?: string,
	url = TEST_PAGE_URL,
) {
	await mockGoogleDocsPage(page, rects, url);
	await page.goto(url);
	await installMockGoogleDocsGeometry(page, rects, annotatedText);
	await page.locator('#harper-google-docs-target').waitFor({ state: 'attached' });
}

async function getRawBridgeText(page: Page) {
	return await page
		.locator('#harper-google-docs-target')
		.evaluate((node) => node.textContent ?? '');
}

async function getNormalizedBridgeText(page: Page) {
	return await page
		.locator('#harper-google-docs-target')
		.evaluate((node) =>
			(node.textContent ?? '').replaceAll('\u00a0', ' ').replace(/\s+/g, ' ').trim(),
		);
}

async function getBridgeSource(page: Page) {
	return await page
		.locator('#harper-google-docs-target')
		.evaluate((node) => node.getAttribute('data-harper-gdocs-source'));
}

test('Google Docs restores spaces around formatted inline words', async ({ page }) => {
	await openMockGoogleDocsPage(page, FORMATTED_WORD_GAP_RECTS, 'not smart enough.');

	await expect
		.poll(() => getNormalizedBridgeText(page), { timeout: 10000 })
		.toBe('not smart enough.');
	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).not.toContain('\n');
});

test('Google Docs formatted rects stay in the same sentence', async ({ page }) => {
	await openMockGoogleDocsPage(page, ITALIC_SHIFTED_RECTS);

	await expect
		.poll(() => getNormalizedBridgeText(page), { timeout: 10000 })
		.toBe('This is an test.');
	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).not.toContain('\n');
});

test('Google Docs does not invent spaces around standalone punctuation rects', async ({ page }) => {
	await openMockGoogleDocsPage(page, PUNCTUATION_BOUNDARY_RECTS, 'not smart enough. But');

	await expect
		.poll(() => getNormalizedBridgeText(page), { timeout: 10000 })
		.toBe('not smart enough. But');
	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).not.toContain(' .');
});

test('Google Docs superscript-like rects do not create a fake line break', async ({ page }) => {
	await openMockGoogleDocsPage(page, SUPERSCRIPT_LIKE_RECTS);

	await expect
		.poll(() => getNormalizedBridgeText(page), { timeout: 10000 })
		.toBe('Testing formatting matters.');
	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).not.toContain('\n');
});

test('Google Docs keeps soft wraps out of the logical bridge text', async ({ page }) => {
	await openMockGoogleDocsPage(
		page,
		WRAPPED_LINE_RECTS,
		'This sentence wraps because the viewport is narrow.',
	);

	await expect
		.poll(() => getRawBridgeText(page), { timeout: 10000 })
		.toBe('This sentence wraps because the viewport is narrow.');
});

test('Google Docs preserves paragraph breaks from annotated text', async ({ page }) => {
	await openMockGoogleDocsPage(
		page,
		PARAGRAPH_BREAK_RECTS,
		'First paragraph.\n\nSecond paragraph.',
	);

	await expect
		.poll(() => getRawBridgeText(page), { timeout: 10000 })
		.toBe('First paragraph.\n\nSecond paragraph.');
});

test('Google Docs keeps numbered list markers with closing parentheses on single list lines', async ({
	page,
}) => {
	await openMockGoogleDocsPage(
		page,
		NUMBERED_PAREN_LIST_RECTS,
		'This paragraph stays sentence case.\n1) This list item should stay sentence case\n2) Another list item should stay sentence case',
	);

	await expect
		.poll(() => getRawBridgeText(page), { timeout: 10000 })
		.toBe(
			'This paragraph stays sentence case.\n1) This list item should stay sentence case\n2) Another list item should stay sentence case',
		);
});

test('Google Docs keeps NBSP-separated formatted inline words in the same sentence', async ({
	page,
}) => {
	await openMockGoogleDocsPage(page, NBSP_FORMATTED_GAP_RECTS, 'not\u00a0smart enough.');

	await expect
		.poll(() => getNormalizedBridgeText(page), { timeout: 10000 })
		.toBe('not smart enough.');
});

test('Google Docs does not invent spaces around em dash rects in fallback mode', async ({
	page,
}) => {
	await openMockGoogleDocsPage(page, EM_DASH_SPLIT_RECTS, '');

	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).toBe('word—word');
});

test('Google Docs keeps wide same-row fallback gaps in a single logical line', async ({ page }) => {
	await openMockGoogleDocsPage(page, WIDE_SAME_ROW_GAP_RECTS, '');

	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).toBe('Name Value');
});

test('Google Docs keeps column-major table rects in logical row order', async ({ page }) => {
	await openMockGoogleDocsPage(page, COLUMN_MAJOR_TABLE_RECTS, COLUMN_MAJOR_TABLE_TEXT);

	await expect.poll(() => getBridgeSource(page), { timeout: 10000 }).toBe('logical');
	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).toBe(COLUMN_MAJOR_TABLE_TEXT);
});

test('Google Docs repairs collapsed wrapped words from malformed logical text', async ({
	page,
}) => {
	await openMockGoogleDocsPage(
		page,
		COLUMN_MAJOR_TABLE_RECTS,
		COLUMN_MAJOR_TABLE_COLLAPSED_WORD_TEXT,
	);

	await expect
		.poll(() => getNormalizedBridgeText(page), { timeout: 10000 })
		.toContain('is available in the early morning before 9 and after 3:30.');
	await expect.poll(() => getRawBridgeText(page), { timeout: 10000 }).not.toContain('earlymorning');
	await expect
		.poll(async () => await page.locator('#harper-highlight').count(), { timeout: 10000 })
		.toBe(0);
});

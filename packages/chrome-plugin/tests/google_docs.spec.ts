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

async function mockGoogleDocsPage(page: Page, rects: MockGoogleDocsRect[]) {
	await page.route(TEST_PAGE_URL, async (route) => {
		await route.fulfill({
			contentType: 'text/html',
			body: buildMockGoogleDocsHtml(rects),
		});
	});
}

async function installMockGoogleDocsGeometry(page: Page, rects: MockGoogleDocsRect[]) {
	await page.evaluate((pageRects) => {
		(
			window as Window & {
				_docs_annotate_getAnnotatedText?: () => Promise<{
					getText: () => string;
					setSelection: () => void;
					getSelection: () => Array<{ start: number; end: number }>;
				}>;
			}
		)._docs_annotate_getAnnotatedText = async () => ({
			getText: () => pageRects.map((rect) => rect.label).join(''),
			setSelection: () => {},
			getSelection: () => [{ start: 0, end: 0 }],
		});
	}, rects);
}

async function openMockGoogleDocsPage(page: Page, rects: MockGoogleDocsRect[]) {
	await mockGoogleDocsPage(page, rects);
	await page.goto(TEST_PAGE_URL);
	await installMockGoogleDocsGeometry(page, rects);
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

test('Google Docs restores spaces around formatted inline words', async ({ page }) => {
	await openMockGoogleDocsPage(page, FORMATTED_WORD_GAP_RECTS);

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
	await openMockGoogleDocsPage(page, PUNCTUATION_BOUNDARY_RECTS);

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

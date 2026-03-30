import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { waitForHarperHighlightCenter } from './testUtils';

type MockGoogleDocsRect = {
	label: string;
	left: number;
	top: number;
	width: number;
	height: number;
	fontCss?: string;
};

const GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS = 20000;

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

async function openMockGoogleDocsPage(
	page: Page,
	url: string,
	rects: MockGoogleDocsRect[],
	annotatedText: string,
) {
	await page.route(url, async (route) => {
		await route.fulfill({
			contentType: 'text/html',
			body: buildMockGoogleDocsHtml(rects),
		});
	});
	await page.goto(url);
	await page.evaluate(({ pageText }) => {
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
	}, { pageText: annotatedText });
	await page.locator('#harper-google-docs-target').waitFor({ state: 'attached' });
}

async function getBridgeSource(page: Page) {
	return await page
		.locator('#harper-google-docs-target')
		.evaluate((node) => node.getAttribute('data-harper-gdocs-source'));
}

test('Google Docs anchors table-cell typo highlights to the right column text', async ({
	page,
}) => {
	const testPageUrl =
		`https://docs.google.com/document/d/harper-table-anchor-${Date.now().toString(36)}/edit`;
	const typoRects: MockGoogleDocsRect[] = [
		{
			label: 'Dana Mills',
			left: 48,
			top: 48,
			width: 88,
			height: 18,
			fontCss: '16px Arial',
		},
		{
			label: 'Emily Stone',
			left: 48,
			top: 120,
			width: 92,
			height: 18,
			fontCss: '16px Arial',
		},
		{
			label: 'writes careful notes for the team.',
			left: 296,
			top: 48,
			width: 224,
			height: 18,
			fontCss: '16px Arial',
		},
		{
			label: 'This is teh plan.',
			left: 296,
			top: 120,
			width: 120,
			height: 18,
			fontCss: '16px Arial',
		},
	];
	const typoText = [
		'Dana Mills',
		'writes careful notes for the team.',
		'Emily Stone',
		'This is teh plan.',
	].join('\n');

	await openMockGoogleDocsPage(page, testPageUrl, typoRects, typoText);

	await expect.poll(() => getBridgeSource(page), { timeout: 10000 }).toBe('logical');
	await expect
		.poll(async () => await page.locator('#harper-highlight').count(), {
			timeout: GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS,
		})
		.toBeGreaterThan(0);

	const center = await waitForHarperHighlightCenter(page, GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS);
	expect(center).not.toBeNull();
	expect(center!.x).toBeGreaterThan(260);
	expect(center!.y).toBeGreaterThan(140);
});

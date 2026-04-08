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

async function installMockGoogleDocsGeometry(page: Page, annotatedText: string) {
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
	await installMockGoogleDocsGeometry(page, annotatedText);
	await page.locator('#harper-google-docs-target').waitFor({ state: 'attached' });
}

async function updateMockGoogleDocsGeometry(
	page: Page,
	rects: MockGoogleDocsRect[],
	annotatedText: string,
) {
	await page.evaluate(
		({ nextRects, pageText }) => {
			const svg = document.querySelector('svg');
			if (!(svg instanceof SVGElement)) {
				throw new Error('Missing mock Google Docs SVG');
			}

			svg.replaceChildren();

			for (const [index, rect] of nextRects.entries()) {
				const node = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
				node.setAttribute('data-rect-index', String(index));
				node.setAttribute('x', String(rect.left));
				node.setAttribute('y', String(rect.top));
				node.setAttribute('width', String(rect.width));
				node.setAttribute('height', String(rect.height));
				node.setAttribute('fill', 'rgba(15, 23, 42, 0.001)');
				node.setAttribute('aria-label', rect.label);
				node.setAttribute('data-font-css', rect.fontCss ?? '');
				svg.appendChild(node);
			}

			(
				window as Window & {
					__harperMockGoogleDocsText?: string;
				}
			).__harperMockGoogleDocsText = pageText;
		},
		{ nextRects: rects, pageText: annotatedText },
	);
}

async function getNormalizedBridgeText(page: Page) {
	return await page
		.locator('#harper-google-docs-target')
		.evaluate((node) =>
			(node.textContent ?? '').replaceAll('\u00a0', ' ').replace(/\s+/g, ' ').trim(),
		);
}

async function startHighlightCountSampling(page: Page) {
	await page.evaluate(() => {
		const win = window as Window & {
			__harperHighlightSamples?: number[];
			__harperHighlightSamplerId?: number;
		};
		win.__harperHighlightSamples = [];

		const sample = () => {
			const count = Array.from(document.querySelectorAll<HTMLElement>('harper-render-box')).reduce(
				(total, host) =>
					total + (host.shadowRoot?.querySelectorAll('#harper-highlight').length ?? 0),
				0,
			);
			win.__harperHighlightSamples?.push(count);
		};

		sample();
		win.__harperHighlightSamplerId = window.setInterval(sample, 40);
	});
}

async function stopHighlightCountSampling(page: Page): Promise<number[]> {
	return await page.evaluate(() => {
		const win = window as Window & {
			__harperHighlightSamples?: number[];
			__harperHighlightSamplerId?: number;
		};

		if (win.__harperHighlightSamplerId != null) {
			window.clearInterval(win.__harperHighlightSamplerId);
			delete win.__harperHighlightSamplerId;
		}

		return [...(win.__harperHighlightSamples ?? [])];
	});
}

test.describe('Google Docs support', () => {
	test('Google Docs keeps existing typo highlights stable while unrelated text is appended (#3122)', async ({
		page,
	}) => {
		const initialText = 'Stable context. This is teh plan.';
		const updatedText = `${initialText} More text.`;
		const testPageUrl = 'CHANGE_ME';
		const initialRects: MockGoogleDocsRect[] = [
			{
				label: 'Stable context. This is ',
				left: 48,
				top: 48,
				width: 168,
				height: 18,
				fontCss: '16px Arial',
			},
			{
				label: 'teh plan.',
				left: 220,
				top: 48,
				width: 64,
				height: 18,
				fontCss: '16px Arial',
			},
		];
		const updatedRects: MockGoogleDocsRect[] = [
			...initialRects,
			{
				label: ' More text.',
				left: 292,
				top: 48,
				width: 72,
				height: 18,
				fontCss: '16px Arial',
			},
		];

		await openMockGoogleDocsPage(page, testPageUrl, initialRects, initialText);

		await expect
			.poll(async () => await page.locator('#harper-highlight').count(), {
				timeout: GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS,
			})
			.toBeGreaterThan(0);

		await startHighlightCountSampling(page);
		await updateMockGoogleDocsGeometry(page, updatedRects, updatedText);

		await expect.poll(() => getNormalizedBridgeText(page), { timeout: 10000 }).toBe(updatedText);
		await expect
			.poll(async () => await page.locator('#harper-highlight').count(), {
				timeout: GOOGLE_DOCS_HIGHLIGHT_TIMEOUT_MS,
			})
			.toBeGreaterThan(0);

		await page.waitForTimeout(400);
		const highlightSamples = await stopHighlightCountSampling(page);

		expect(highlightSamples.some((count) => count > 0)).toBe(true);
		expect(highlightSamples).not.toContain(0);
	});
});

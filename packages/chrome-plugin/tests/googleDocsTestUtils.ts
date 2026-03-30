import type { Page } from '@playwright/test';
import { expect } from './fixtures';

export const LIVE_GOOGLE_DOCS_URL =
	'https://docs.google.com/document/d/1ybGsBpMShQhXgmAhTmioVeQbDBf1WY_GrWmIODf0wQ4/edit?usp=sharing';

const GOOGLE_DOCS_RESET_SENTINEL = 'Harper reset sentinel';

type GoogleDocsRenderedState = {
	annotatedText: string;
	bridgeText: string;
	rectFonts: string[];
	rectLabels: string[];
	highlightCount: number;
	lineCount: number;
};

function normalizeWhitespace(text: string): string {
	return text.replaceAll('\u00a0', ' ').replace(/\s+/g, ' ').trim();
}

function normalizeAnnotatedText(text: string): string {
	return text.replace(/^\u0003/, '').replace(/\n$/, '');
}

export function normalizeGoogleDocsBridgeText(text: string): string {
	return text.replaceAll('\u00a0', ' ').trimEnd();
}

export async function openLiveGoogleDoc(page: Page) {
	await page.goto(LIVE_GOOGLE_DOCS_URL, {
		waitUntil: 'domcontentloaded',
		timeout: 120000,
	});

	await page.locator('#docs-editor').waitFor({ state: 'visible', timeout: 60000 });

	await expect
		.poll(
			async () => {
				return await page.evaluate(async () => {
					if (typeof window._docs_annotate_getAnnotatedText !== 'function') {
						return false;
					}

					try {
						const annotated = await window._docs_annotate_getAnnotatedText();
						return Boolean(
							annotated?.getText &&
								document.querySelector('#harper-google-docs-target') &&
								document.querySelector('#harper-google-docs-main-world-bridge'),
						);
					} catch {
						return false;
					}
				});
			},
			{ timeout: 60000 },
		)
		.toBe(true);
}

export async function focusGoogleDocsEditor(page: Page) {
	await page.locator('#docs-editor').click({
		position: { x: 260, y: 180 },
		timeout: 30000,
	});
	await page.waitForTimeout(750);
}

export async function setGoogleDocsFontSize(page: Page, fontSizePoints: number) {
	await page.keyboard.press('Escape');
	await page.waitForTimeout(200);

	const fontSizeInput = page.locator('input[aria-label="Font size"]').first();
	await expect(fontSizeInput).toBeVisible({ timeout: 10000 });
	await fontSizeInput.click({ clickCount: 3 });
	await page.waitForTimeout(150);
	await fontSizeInput.fill(String(fontSizePoints));
	await fontSizeInput.press('Enter');
	await page.waitForTimeout(750);

	await focusGoogleDocsEditor(page);
}

export async function moveGoogleDocsCursorToEnd(page: Page) {
	await page.evaluate(async () => {
		const annotated = await window._docs_annotate_getAnnotatedText?.();
		const source = annotated?.getText?.() ?? '';
		annotated?.setSelection?.(source.length, source.length);
	});
	await page.waitForTimeout(500);
}

async function getGoogleDocsRenderedState(page: Page): Promise<GoogleDocsRenderedState> {
	return await page.evaluate(async () => {
		const annotated = await window._docs_annotate_getAnnotatedText();
		const rects = Array.from(
			document.querySelectorAll<SVGRectElement>('.kix-appview-editor rect[aria-label]'),
		);
		const lineTops = new Set(
			rects
				.map((rect) => Math.round(rect.getBoundingClientRect().top))
				.filter((top) => Number.isFinite(top)),
		);

		return {
			annotatedText: annotated.getText(),
			bridgeText: document.querySelector('#harper-google-docs-target')?.textContent ?? '',
			rectFonts: rects.map((rect) => rect.getAttribute('data-font-css') ?? ''),
			rectLabels: rects.map((rect) => rect.getAttribute('aria-label') ?? ''),
			highlightCount: document.querySelectorAll('#harper-highlight').length,
			lineCount: lineTops.size,
		};
	});
}

export async function getGoogleDocsAnnotatedText(page: Page): Promise<string> {
	const state = await getGoogleDocsRenderedState(page);
	return normalizeAnnotatedText(state.annotatedText);
}

export async function getGoogleDocsBridgeText(page: Page): Promise<string> {
	const state = await getGoogleDocsRenderedState(page);
	return normalizeGoogleDocsBridgeText(state.bridgeText);
}

export async function getGoogleDocsNormalizedBridgeText(page: Page): Promise<string> {
	return normalizeWhitespace(await getGoogleDocsBridgeText(page));
}

export async function getGoogleDocsHighlightCount(page: Page): Promise<number> {
	const state = await getGoogleDocsRenderedState(page);
	return state.highlightCount;
}

export async function getGoogleDocsVisualLineCount(page: Page): Promise<number> {
	const state = await getGoogleDocsRenderedState(page);
	return state.lineCount;
}

async function clearGoogleDocsDocument(page: Page) {
	await focusGoogleDocsEditor(page);
	await page.keyboard.press('Escape');
	await page.waitForTimeout(200);
	await page.evaluate(async () => {
		const annotated = await window._docs_annotate_getAnnotatedText?.();
		const source = annotated?.getText?.() ?? '';
		annotated?.setSelection?.(0, source.length);
	});
	await page.waitForTimeout(300);
	await page.keyboard.press('Control+A');
	await page.waitForTimeout(200);
	await page.keyboard.press('Control+A');
	await page.waitForTimeout(300);
	await page.keyboard.press('Backspace');
	await page.waitForTimeout(1000);
}

async function clearGoogleDocsDocumentCompletely(page: Page) {
	for (let attempt = 0; attempt < 3; attempt += 1) {
		await clearGoogleDocsDocument(page);

		try {
			await expect.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 10000 }).toBe('');
			return;
		} catch {
			await focusGoogleDocsEditor(page);
			await page.keyboard.press('Escape');
			await page.waitForTimeout(200);
			await page.evaluate(async () => {
				const annotated = await window._docs_annotate_getAnnotatedText?.();
				const source = annotated?.getText?.() ?? '';
				annotated?.setSelection?.(0, source.length);
			});
			await page.waitForTimeout(300);
			await page.keyboard.press('Control+A');
			await page.waitForTimeout(200);
			await page.keyboard.press('Control+A');
			await page.waitForTimeout(300);
			await page.keyboard.press('Delete');
			await page.waitForTimeout(1000);
		}
	}

	throw new Error('Failed to clear the Google Docs document');
}

async function typeGoogleDocsText(page: Page, text: string) {
	const lines = text.split('\n');

	for (let i = 0; i < lines.length; i += 1) {
		if (lines[i].length > 0) {
			await page.keyboard.type(lines[i], { delay: 20 });
		}

		if (i < lines.length - 1) {
			await page.keyboard.press('Enter');
			await page.waitForTimeout(150);
		}
	}

	await page.waitForTimeout(1500);
}

async function openGoogleDocsInsertTablePicker(page: Page) {
	await page.getByRole('menuitem', { name: 'Insert' }).click();
	await page.waitForTimeout(300);
	await page.locator('.goog-menuitem').filter({ hasText: 'Table' }).first().hover();
	await page.waitForTimeout(500);
}

export async function insertGoogleDocsTable(page: Page, rows: number, columns: number) {
	if (rows < 1 || columns < 1) {
		throw new Error('Google Docs tables must have at least one row and one column');
	}

	await focusGoogleDocsEditor(page);
	await openGoogleDocsInsertTablePicker(page);

	const grid = page.locator('.goog-dimension-picker-unhighlighted');
	await expect(grid).toBeVisible({ timeout: 10000 });
	const gridBox = await grid.boundingBox();
	if (!gridBox) {
		throw new Error('Could not find the Google Docs table picker grid');
	}

	const cellSize = gridBox.width / 11;
	const targetX = gridBox.x + (columns - 0.5) * cellSize;
	const targetY = gridBox.y + (rows - 0.5) * cellSize;

	await page.mouse.move(targetX, targetY);
	await page.waitForTimeout(300);
	await page.mouse.click(targetX, targetY);
	await page.waitForTimeout(1000);
}

export async function typeGoogleDocsTableCells(page: Page, rows: string[][]) {
	const columnCount = rows[0]?.length ?? 0;
	if (columnCount === 0 || rows.some((row) => row.length !== columnCount)) {
		throw new Error('Google Docs table rows must all have the same number of cells');
	}

	const cells = rows.flat();
	for (let cellIndex = 0; cellIndex < cells.length; cellIndex += 1) {
		if (cells[cellIndex].length > 0) {
			await page.keyboard.type(cells[cellIndex], { delay: 20 });
		}

		if (cellIndex < cells.length - 1) {
			await page.keyboard.press('Tab');
			await page.waitForTimeout(250);
		}
	}

	await page.waitForTimeout(1500);
}

export async function appendGoogleDocsTable(page: Page, rows: string[][]) {
	if (rows.length === 0) {
		throw new Error('Google Docs tables must include at least one row');
	}

	await moveGoogleDocsCursorToEnd(page);
	await focusGoogleDocsEditor(page);
	await page.keyboard.press('Enter');
	await page.waitForTimeout(500);
	await insertGoogleDocsTable(page, rows.length, rows[0].length);
	await typeGoogleDocsTableCells(page, rows);
}

async function ensurePlainGoogleDocsInsertionMode(page: Page) {
	for (let attempt = 0; attempt < 3; attempt += 1) {
		await clearGoogleDocsDocument(page);
		await typeGoogleDocsText(page, GOOGLE_DOCS_RESET_SENTINEL);

		try {
			await expect
				.poll(() => getGoogleDocsNormalizedBridgeText(page), { timeout: 15000 })
				.toContain(GOOGLE_DOCS_RESET_SENTINEL);
		} catch {
			continue;
		}

		const state = await getGoogleDocsRenderedState(page);
		const bridgeText = normalizeGoogleDocsBridgeText(state.bridgeText);
		const hasListMarker = /^([●•◦▪‣]|\d+\.)\s/.test(bridgeText);
		const hasUnexpectedFont = state.rectFonts.some((font) => /italic|bold/.test(font));

		if (!hasListMarker && !hasUnexpectedFont) {
			return;
		}

		await focusGoogleDocsEditor(page);
		await page.keyboard.press('Control+A');
		await page.waitForTimeout(300);

		if (hasListMarker) {
			await page.keyboard.press('Control+Shift+8');
			await page.waitForTimeout(1000);
		}

		if (hasUnexpectedFont) {
			await page.keyboard.press('Control+\\\\');
			await page.waitForTimeout(1000);
		}
	}

	throw new Error('Failed to reset Google Docs formatting state');
}

export async function replaceGoogleDocsDocumentText(page: Page, text: string) {
	await ensurePlainGoogleDocsInsertionMode(page);
	await clearGoogleDocsDocumentCompletely(page);

	if (text.length > 0) {
		await typeGoogleDocsText(page, text);
	}

	await expect.poll(() => getGoogleDocsAnnotatedText(page), { timeout: 20000 }).toBe(text);
}

async function selectGoogleDocsRange(page: Page, start: number, end: number) {
	await page.evaluate(
		async ({ selectionStart, selectionEnd }) => {
			const annotated = await window._docs_annotate_getAnnotatedText();
			annotated.setSelection(selectionStart, selectionEnd);
		},
		{
			selectionStart: start,
			selectionEnd: end,
		},
	);
	await page.waitForTimeout(500);
}

export async function selectGoogleDocsText(page: Page, text: string, occurrence = 0) {
	const selection = await page.evaluate(
		async ({ needle, targetOccurrence }) => {
			const annotated = await window._docs_annotate_getAnnotatedText();
			const source = annotated.getText();
			let start = -1;
			let fromIndex = 0;

			for (let i = 0; i <= targetOccurrence; i += 1) {
				start = source.indexOf(needle, fromIndex);
				if (start < 0) {
					return null;
				}
				fromIndex = start + needle.length;
			}

			return { start, end: start + needle.length };
		},
		{
			needle: text,
			targetOccurrence: occurrence,
		},
	);

	if (selection == null) {
		throw new Error(`Could not find "${text}" in Google Docs text`);
	}

	await selectGoogleDocsRange(page, selection.start, selection.end);
}

export async function selectGoogleDocsFromTextToEnd(page: Page, text: string) {
	const selection = await page.evaluate(async (needle) => {
		const annotated = await window._docs_annotate_getAnnotatedText();
		const source = annotated.getText();
		const start = source.indexOf(needle);
		if (start < 0) {
			return null;
		}

		return { start, end: source.length };
	}, text);

	if (selection == null) {
		throw new Error(`Could not find "${text}" in Google Docs text`);
	}

	await selectGoogleDocsRange(page, selection.start, selection.end);
}

export async function hasGoogleDocsRectWithFontContainingText(
	page: Page,
	text: string,
	fontPattern: RegExp,
): Promise<boolean> {
	const state = await getGoogleDocsRenderedState(page);
	const normalizedNeedle = normalizeWhitespace(text);

	return state.rectLabels.some((label, index) => {
		const normalizedLabel = normalizeWhitespace(label);
		const fontCss = state.rectFonts[index] ?? '';
		return normalizedLabel.includes(normalizedNeedle) && fontPattern.test(fontCss);
	});
}

export async function applyItalicToGoogleDocsText(page: Page, text: string, occurrence = 0) {
	await selectGoogleDocsText(page, text, occurrence);
	await page.keyboard.press('Control+I');

	await expect
		.poll(() => hasGoogleDocsRectWithFontContainingText(page, text, /italic/i), {
			timeout: 15000,
		})
		.toBe(true);
}

export async function applyBulletedListToGoogleDocsText(page: Page, text: string) {
	await selectGoogleDocsFromTextToEnd(page, text);
	await page.keyboard.press('Control+Shift+8');

	await expect.poll(() => getGoogleDocsBridgeText(page), { timeout: 15000 }).toContain(`● ${text}`);
}

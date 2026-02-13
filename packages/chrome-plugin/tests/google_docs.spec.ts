import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';

const GOOGLE_DOC_URL =
	'https://docs.google.com/document/d/1ybGsBpMShQhXgmAhTmioVeQbDBf1WY_GrWmIODf0wQ4/edit?usp=sharing';
const RUN_GOOGLE_DOCS_TESTS = process.env.HARPER_E2E_GOOGLE_DOCS === '1';

test.skip(
	!RUN_GOOGLE_DOCS_TESTS,
	'Google Docs tests require network access and a live editable document. Set HARPER_E2E_GOOGLE_DOCS=1 to run.',
);

async function getGoogleDocText(page: Page) {
	return page.evaluate(async () => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') {
			return null;
		}

		const annotated = await getAnnotatedText();
		if (!annotated || typeof annotated.getText !== 'function') {
			return null;
		}

		return annotated.getText() as string;
	});
}

async function replaceDocumentContent(page: Page, line: string) {
	await page.locator('.kix-appview-editor').click();
	await page.keyboard.press('ControlOrMeta+A');
	await page.keyboard.type(line);
}

test('Google Docs: Harper can read lintable text', async ({ page }) => {
	const token = `harper-gdocs-read-${Date.now()}`;
	const input = `This is an test ${token}`;

	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);
});

test('Google Docs: Harper can write a suggestion back into the document', async ({ page }) => {
	const token = `harper-gdocs-write-${Date.now()}`;
	const input = `This is an test ${token}`;
	const corrected = `This is a test ${token}`;

	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const fullText = (await getGoogleDocText(page)) ?? '';
	const start = fullText.indexOf(`an test ${token}`);
	expect(start).toBeGreaterThanOrEqual(0);
	await page.evaluate(
		({ start }) => {
			document.dispatchEvent(
				new CustomEvent('harper:gdocs:replace', {
					detail: {
						start,
						end: start + 2,
						replacementText: 'a',
					},
				}),
			);
		},
		{ start },
	);

	await expect
		.poll(async () => {
			const text = await getGoogleDocText(page);
			return text ?? '';
		})
			.toContain(corrected);
});

test('Google Docs: highlight appears near linted text', async ({ page }) => {
	const token = `harper-gdocs-position-${Date.now()}`;
	const input = `This is an test ${token}`;

	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const caretRect = await page.evaluate(async (token) => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') return null;
		const annotated = await getAnnotatedText();
		const text = annotated.getText() as string;
		const start = text.indexOf(`an test ${token}`);
		if (start < 0) return null;
		annotated.setSelection(start, start);
		const caret = document.querySelector('.kix-cursor-caret');
		const rect = caret?.getBoundingClientRect();
		if (!rect) return null;
		return { x: rect.x, y: rect.y };
	}, token);
	expect(caretRect).not.toBeNull();

	const highlightBox = await page.locator('#harper-highlight').first().boundingBox();
	expect(highlightBox).not.toBeNull();
	expect(Math.abs((highlightBox?.x ?? 0) - (caretRect?.x ?? 0))).toBeLessThan(120);
	expect(Math.abs((highlightBox?.y ?? 0) - (caretRect?.y ?? 0))).toBeLessThan(60);
});

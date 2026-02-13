import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';

const GOOGLE_DOC_URL =
	'https://docs.google.com/document/d/1ybGsBpMShQhXgmAhTmioVeQbDBf1WY_GrWmIODf0wQ4/edit?usp=sharing';
const RUN_GOOGLE_DOCS_TESTS = process.env.HARPER_E2E_GOOGLE_DOCS === '1';

test.skip(
	!RUN_GOOGLE_DOCS_TESTS,
	'Google Docs tests require network access and a live editable document. Set HARPER_E2E_GOOGLE_DOCS=1 to run.',
);
test.describe.configure({ mode: 'serial' });

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

async function getGoogleDocsEditorScrollTop(page: Page) {
	return page.evaluate(() => {
		const editor = document.querySelector('.kix-appview-editor') as HTMLElement | null;
		return editor?.scrollTop ?? 0;
	});
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

	const highlightBoxes = await page.locator('#harper-highlight').evaluateAll((nodes) =>
		nodes
			.map((node) => {
				const rect = node.getBoundingClientRect();
				return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
			})
			.filter((rect) => rect.width > 0 && rect.height > 0),
	);
	expect(highlightBoxes.length).toBeGreaterThan(0);

	const closest = highlightBoxes.reduce(
		(best, box) => {
			const dx = Math.abs(box.x - (caretRect?.x ?? 0));
			const dy = Math.abs(box.y - (caretRect?.y ?? 0));
			const score = dx + dy;
			return score < best.score ? { dx, dy, score } : best;
		},
		{ dx: Number.POSITIVE_INFINITY, dy: Number.POSITIVE_INFINITY, score: Number.POSITIVE_INFINITY },
	);

	expect(closest.dx).toBeLessThan(180);
	expect(closest.dy).toBeLessThan(90);
});

test('Google Docs: highlight host mounts on document body', async ({ page }) => {
	const token = `harper-gdocs-host-${Date.now()}`;
	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
	await replaceDocumentContent(page, `This is an test ${token}`);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const mountedOnBody = await page.evaluate(() => {
		const host = document.querySelector('#harper-highlight-host');
		return host?.parentElement === document.body;
	});
	expect(mountedOnBody).toBe(true);
});

test('Google Docs: selection does not grow over time', async ({ page }) => {
	const token = `harper-gdocs-selection-${Date.now()}`;
	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
	await replaceDocumentContent(page, `This is an test ${token}`);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const before = await page.evaluate(async (token) => {
		const annotated = await (window as any)._docs_annotate_getAnnotatedText();
		const text = annotated.getText() as string;
		const start = text.indexOf(`an test ${token}`);
		annotated.setSelection(start, start);
		const sel = annotated.getSelection()[0];
		return { start: sel.start, end: sel.end };
	}, token);

	await page.waitForTimeout(2500);

	const after = await page.evaluate(async () => {
		const annotated = await (window as any)._docs_annotate_getAnnotatedText();
		const sel = annotated.getSelection()[0];
		return { start: sel.start, end: sel.end };
	});

	expect(after.start).toBe(before.start);
	expect(after.end).toBe(before.end);
});

test('Google Docs: scrolling does not snap back upward', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-scroll-${Date.now()}`;
	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
	const longText = [`This is an test ${token}`]
		.concat(Array.from({ length: 80 }, (_, i) => `line ${i} ${token}`))
		.join('\n');
	await replaceDocumentContent(page, longText);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const initial = await getGoogleDocsEditorScrollTop(page);
	for (let i = 0; i < 12; i++) {
		await page.keyboard.press('PageDown');
	}
	await page.waitForTimeout(500);
	const scrolled = await getGoogleDocsEditorScrollTop(page);

	await page.waitForTimeout(2500);
	const afterWait = await getGoogleDocsEditorScrollTop(page);
	expect(afterWait).toBeGreaterThan(scrolled - 30);
});

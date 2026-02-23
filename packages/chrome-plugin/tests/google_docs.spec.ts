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

async function getGoogleDocsTargetText(page: Page) {
	return page.evaluate(() => {
		const target = document.getElementById('harper-google-docs-target');
		return target?.textContent ?? null;
	});
}

function normalizeGoogleDocText(text: string): string {
	return text.replace(/\u0003/g, '').replace(/\r\n/g, '\n').trimEnd();
}

async function replaceGoogleDocRange(
	page: Page,
	start: number,
	end: number,
	replacementText: string,
	sourceText?: string,
): Promise<void> {
	const safeSource = sourceText ?? '';
	const safeStart = Math.max(0, Math.min(start, safeSource.length));
	const safeEnd = Math.max(safeStart, Math.min(end, safeSource.length));
	const expectedText = safeSource.slice(safeStart, safeEnd);
	const beforeContext = safeSource.slice(Math.max(0, safeStart - 64), safeStart);
	const afterContext = safeSource.slice(safeEnd, Math.min(safeSource.length, safeEnd + 64));

	await page.evaluate(
		({ start, end, replacementText, expectedText, beforeContext, afterContext }) => {
			document.dispatchEvent(
				new CustomEvent('harper:gdocs:replace', {
					detail: {
						start,
						end,
						replacementText,
						expectedText,
						beforeContext,
						afterContext,
					},
				}),
			);
		},
		{ start, end, replacementText, expectedText, beforeContext, afterContext },
	);
}

async function replaceDocumentContent(page: Page, line: string) {
	await page.locator('.kix-appview-editor').click();
	await page.keyboard.press('ControlOrMeta+A');
	await page.keyboard.type(line);
}

async function openGoogleDoc(page: Page) {
	await page.goto('about:blank');
	await page.goto(GOOGLE_DOC_URL);
	await page.locator('.kix-appview-editor').waitFor({ state: 'visible' });
}

async function getGoogleDocsEditorScrollTop(page: Page) {
	return page.evaluate(() => {
		const editor = document.querySelector('.kix-appview-editor') as HTMLElement | null;
		return editor?.scrollTop ?? 0;
	});
}

async function getGoogleDocSelection(page: Page) {
	return page.evaluate(async () => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') return null;
		const annotated = await getAnnotatedText();
		const sel = annotated?.getSelection?.()?.[0];
		if (!sel) return null;

		const asNum = (v: unknown) => {
			const n = Number(v);
			return Number.isFinite(n) ? n : null;
		};

		return {
			start: asNum(sel.start),
			end: asNum(sel.end),
			anchor: asNum(sel.anchor),
			focus: asNum(sel.focus),
			base: asNum(sel.base),
			extent: asNum(sel.extent),
		};
	});
}

async function setGoogleDocSelection(page: Page, start: number, end: number) {
	await page.evaluate(
		async ({ start, end }) => {
			const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
			if (typeof getAnnotatedText !== 'function') return;
			const annotated = await getAnnotatedText();
			annotated?.setSelection?.(start, end);
		},
		{ start, end },
	);
}

async function getNeedleRange(page: Page, needle: string) {
	return page.evaluate(async (needle) => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') return null;
		const annotated = await getAnnotatedText();
		const text = annotated?.getText?.() as string;
		if (!text) return null;
		const start = text.indexOf(needle);
		if (start < 0) return null;
		return { start, end: start + needle.length };
	}, needle);
}

async function getCaretRectForNeedle(page: Page, needle: string) {
	return page.evaluate(async (needle) => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') return null;
		const annotated = await getAnnotatedText();
		const text = annotated.getText() as string;
		const start = text.indexOf(needle);
		if (start < 0) return null;
		annotated.setSelection(start, start);
		const caret = document.querySelector('.kix-cursor-caret');
		const rect = caret?.getBoundingClientRect();
		if (!rect) return null;
		return { x: rect.x, y: rect.y, start };
	}, needle);
}

async function getVisibleHighlightBoxes(page: Page) {
	return page.locator('#harper-highlight').evaluateAll((nodes) =>
		nodes
			.map((node) => {
				const rect = node.getBoundingClientRect();
				return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
			})
			.filter((rect) => rect.width > 0 && rect.height > 0),
	);
}

async function getClosestHighlightToNeedle(page: Page, needle: string) {
	const caretRect = await getCaretRectForNeedle(page, needle);
	if (!caretRect) {
		return null;
	}

	const boxes = await getVisibleHighlightBoxes(page);
	if (boxes.length === 0) {
		return null;
	}

	return getClosestBoxDistance(boxes, { x: caretRect.x, y: caretRect.y });
}

function getClosestBoxDistance(
	boxes: { x: number; y: number; width: number; height: number }[],
	point: { x: number; y: number },
) {
	return boxes.reduce(
		(best, box) => {
			const dx = Math.abs(box.x - point.x);
			const dy = Math.abs(box.y - point.y);
			const score = dx + dy;
			return score < best.score ? { dx, dy, score } : best;
		},
		{ dx: Number.POSITIVE_INFINITY, dy: Number.POSITIVE_INFINITY, score: Number.POSITIVE_INFINITY },
	);
}

test('Google Docs: Harper can read lintable text', async ({ page }) => {
	const token = `harper-gdocs-read-${Date.now()}`;
	const input = `This is an test ${token}`;

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);
	await expect
		.poll(async () => ((await getGoogleDocText(page)) ?? '').includes(`an test ${token}`), {
			timeout: 20000,
		})
		.toBeTruthy();

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);
	await expect
		.poll(
			async () =>
				page.evaluate(() => {
					return (
						document.getElementById('harper-google-docs-main-world-bridge') != null &&
						document.getElementById('harper-google-docs-target') != null
					);
				}),
			{ timeout: 10000 },
		)
		.toBeTruthy();
});

test('Google Docs: bridge target preserves paragraph breaks and source parity', async ({ page }) => {
	const token = `harper-gdocs-linebreak-${Date.now()}`;
	const firstLine = `Firstlineword${token}`;
	const secondLine = `This is an test ${token}`;

	await openGoogleDoc(page);
	await replaceDocumentContent(page, `${firstLine}\n${secondLine}`);

	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocsTargetText(page)) ?? ''), {
			timeout: 20000,
		})
		.toContain(`\n${secondLine}`);
	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocsTargetText(page)) ?? ''), {
			timeout: 20000,
		})
		.not.toContain(`${firstLine}${secondLine}`);
	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocText(page)) ?? ''), {
			timeout: 20000,
		})
		.toContain(`\n${secondLine}`);
	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocText(page)) ?? ''), {
			timeout: 20000,
		})
		.toContain(`\n${secondLine}`);
});

test('Google Docs: Harper can write a suggestion back into the document', async ({ page }) => {
	const token = `harper-gdocs-write-${Date.now()}`;
	const input = `This is an test ${token}`;
	const corrected = `This is a test ${token}`;

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);
	await expect
		.poll(async () => ((await getGoogleDocText(page)) ?? '').includes(`an test ${token}`), {
			timeout: 20000,
		})
		.toBeTruthy();

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

test('Google Docs: replace offsets derived from bridge text stay aligned across paragraphs', async ({
	page,
}) => {
	const token = `harper-gdocs-replace-alignment-${Date.now()}`;
	const firstLine = `PrefixLine${token}`;
	const secondLine = `This is an test ${token}`;
	const expected = `${firstLine}\nThis is a test ${token}`;

	await openGoogleDoc(page);
	await replaceDocumentContent(page, `${firstLine}\n${secondLine}`);

	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocsTargetText(page)) ?? ''), {
			timeout: 20000,
		})
		.toContain(`\n${secondLine}`);
	await expect
		.poll(
			async () => {
				const raw = (await getGoogleDocsTargetText(page)) ?? '';
				return raw.indexOf(`an test ${token}`);
			},
			{ timeout: 20000 },
		)
		.toBeGreaterThanOrEqual(0);

	const targetText = (await getGoogleDocsTargetText(page)) ?? '';
	const targetStart = targetText.indexOf(`an test ${token}`);
	expect(targetStart).toBeGreaterThanOrEqual(0);

	await replaceGoogleDocRange(page, targetStart, targetStart + 2, 'a', targetText);

	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocText(page)) ?? ''), {
			timeout: 20000,
		})
		.toContain(expected);
});

test('Google Docs: highlight appears near linted text', async ({ page }) => {
	const token = `harper-gdocs-position-${Date.now()}`;
	const input = `This is an test ${token}`;

	await openGoogleDoc(page);
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

test('Google Docs: highlight host mounts on editor container', async ({ page }) => {
	const token = `harper-gdocs-host-${Date.now()}`;
	await openGoogleDoc(page);
	await replaceDocumentContent(page, `This is an test ${token}`);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const mountedOnEditor = await page.evaluate(() => {
		const host = document.querySelector('#harper-highlight-host');
		const editor = document.querySelector('.kix-appview-editor');
		return host?.parentElement === editor;
	});
	expect(mountedOnEditor).toBe(true);
});

test('Google Docs: selection does not grow over time', async ({ page }) => {
	const token = `harper-gdocs-selection-${Date.now()}`;
	await openGoogleDoc(page);
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
	await openGoogleDoc(page);
	const longText = [`This is an test ${token}`]
		.concat(Array.from({ length: 60 }, (_, i) => `line ${i} ${token}`))
		.join('\n');
	await replaceDocumentContent(page, longText);
	await page.waitForTimeout(1200);
	await page.evaluate(() => {
		const editor = document.querySelector('.kix-appview-editor') as HTMLElement | null;
		if (editor) editor.scrollTop = 0;
	});
	await page.waitForTimeout(250);

	const initial = await getGoogleDocsEditorScrollTop(page);
	await page.evaluate(() => {
		const editor = document.querySelector('.kix-appview-editor') as HTMLElement | null;
		if (!editor) return;
		editor.scrollTop = editor.scrollTop + 1200;
	});
	await page.waitForTimeout(300);
	const scrolled = await getGoogleDocsEditorScrollTop(page);
	expect(scrolled).toBeGreaterThan(200);

	await page.waitForTimeout(2500);
	const afterWait = await getGoogleDocsEditorScrollTop(page);
	expect(afterWait).toBeGreaterThan(80);
});

test('Google Docs: scrolling does not reset after several seconds', async ({ page }) => {
	test.setTimeout(120000);
	const token = `harper-gdocs-scroll-late-${Date.now()}`;
	await openGoogleDoc(page);
	const longText = [`This is an test ${token}`]
		.concat(Array.from({ length: 120 }, (_, i) => `long line ${i} ${token}`))
		.join('\n');
	await replaceDocumentContent(page, longText);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 20000 })
		.toBeGreaterThan(0);

	await page.evaluate(() => {
		const editor = document.querySelector('.kix-appview-editor') as HTMLElement | null;
		if (editor) editor.scrollTop = 0;
	});
	await page.waitForTimeout(500);

	await page.evaluate(() => {
		const editor = document.querySelector('.kix-appview-editor') as HTMLElement | null;
		if (!editor) return;
		editor.scrollTop = editor.scrollTop + 2200;
	});
	await page.waitForTimeout(600);

	const scrolled = await getGoogleDocsEditorScrollTop(page);
	expect(scrolled).toBeGreaterThan(500);

	// Allow enough time for delayed snap-back regressions to manifest.
	await page.waitForTimeout(9000);

	const afterDelay = await getGoogleDocsEditorScrollTop(page);
	expect(afterDelay).toBeGreaterThan(400);
});

test('Google Docs: left-to-right selection persists until user interaction', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-sel-ltr-${Date.now()}`;
	const needle = `an test ${token}`;
	await openGoogleDoc(page);
	await replaceDocumentContent(
		page,
		[`Clean line ${token}`, `This is ${needle}`, `Another line ${token}`].join('\n'),
	);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 20000 })
		.toBeGreaterThan(0);

	const range = await getNeedleRange(page, needle);
	expect(range).not.toBeNull();
	await setGoogleDocSelection(page, range!.start, range!.end);

	const before = await getGoogleDocSelection(page);
	expect(before).not.toBeNull();
	expect(before?.start).not.toBeNull();
	expect(before?.end).not.toBeNull();

	await page.waitForTimeout(5000);

	const afterWait = await getGoogleDocSelection(page);
	expect(afterWait).toEqual(before);

	// User interaction should be the next event that changes selection.
	await page.locator('.kix-appview-editor').click();
	await page.keyboard.press('ArrowRight');

	await expect
		.poll(async () => {
			const now = await getGoogleDocSelection(page);
			return now?.start !== before?.start || now?.end !== before?.end;
		})
		.toBeTruthy();
});

test('Google Docs: right-to-left selection persists until user interaction', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-sel-rtl-${Date.now()}`;
	const needle = `an test ${token}`;
	await openGoogleDoc(page);
	await replaceDocumentContent(
		page,
		[`Start ${token}`, `This is ${needle}`, `Tail ${token}`].join('\n'),
	);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 20000 })
		.toBeGreaterThan(0);

	const range = await getNeedleRange(page, needle);
	expect(range).not.toBeNull();
	await setGoogleDocSelection(page, range!.end, range!.start);

	const before = await getGoogleDocSelection(page);
	expect(before).not.toBeNull();
	expect(before?.start).not.toBeNull();
	expect(before?.end).not.toBeNull();

	await page.waitForTimeout(5000);

	const afterWait = await getGoogleDocSelection(page);
	expect(afterWait).toEqual(before);

	await page.locator('.kix-appview-editor').click();
	await page.keyboard.press('ArrowLeft');

	await expect
		.poll(async () => {
			const now = await getGoogleDocSelection(page);
			return now?.start !== before?.start || now?.end !== before?.end;
		})
		.toBeTruthy();
});

test('Google Docs: highlight appears near second-line lint', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-second-line-${Date.now()}`;
	const lineWithLint = `This is an test ${token}`;
	const input = [`This line is clean ${token}`, lineWithLint, `Another clean line ${token}`].join(
		'\n',
	);

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);
	await expect
		.poll(
			async () => {
				const closest = await getClosestHighlightToNeedle(page, `an test ${token}`);
				return closest?.dx ?? Number.POSITIVE_INFINITY;
			},
			{ timeout: 15000 },
		)
		.toBeLessThan(180);
	await expect
		.poll(
			async () => {
				const closest = await getClosestHighlightToNeedle(page, `an test ${token}`);
				return closest?.dy ?? Number.POSITIVE_INFINITY;
			},
			{ timeout: 15000 },
		)
		.toBeLessThanOrEqual(140);
});

test('Google Docs: highlight appears near third-line lint', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-third-line-${Date.now()}`;
	const input = [
		`This line is clean ${token}`,
		`Still clean here ${token}`,
		`This is an test ${token}`,
	].join('\n');

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);
	await expect
		.poll(
			async () => {
				const closest = await getClosestHighlightToNeedle(page, `an test ${token}`);
				return closest?.dx ?? Number.POSITIVE_INFINITY;
			},
			{ timeout: 15000 },
		)
		.toBeLessThan(180);
	await expect
		.poll(
			async () => {
				const closest = await getClosestHighlightToNeedle(page, `an test ${token}`);
				return closest?.dy ?? Number.POSITIVE_INFINITY;
			},
			{ timeout: 15000 },
		)
		.toBeLessThanOrEqual(140);
});

test('Google Docs: highlight stays near text for at least 15 seconds', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-stability-${Date.now()}`;
	const input = [
		`Clean line ${token}`,
		`This is an test ${token}`,
		`Another clean line ${token}`,
	].join('\n');

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const initialCaret = await getCaretRectForNeedle(page, `an test ${token}`);
	expect(initialCaret).not.toBeNull();
	const initialBoxes = await getVisibleHighlightBoxes(page);
	expect(initialBoxes.length).toBeGreaterThan(0);
	const initialClosest = getClosestBoxDistance(initialBoxes, {
		x: initialCaret?.x ?? 0,
		y: initialCaret?.y ?? 0,
	});
	expect(initialClosest.dx).toBeLessThan(180);
	expect(initialClosest.dy).toBeLessThanOrEqual(140);

	await page.waitForTimeout(16000);

	const laterCaret = await getCaretRectForNeedle(page, `an test ${token}`);
	expect(laterCaret).not.toBeNull();
	const laterBoxes = await getVisibleHighlightBoxes(page);
	expect(laterBoxes.length).toBeGreaterThan(0);
	const laterClosest = getClosestBoxDistance(laterBoxes, {
		x: laterCaret?.x ?? 0,
		y: laterCaret?.y ?? 0,
	});
	expect(laterClosest.dx).toBeLessThan(180);
	expect(laterClosest.dy).toBeLessThanOrEqual(140);
});

test('Google Docs: line geometry differs between repeated lint phrases', async ({ page }) => {
	test.setTimeout(90000);
	const token = `harper-gdocs-multi-line-${Date.now()}`;
	const input = [
		`This is an test ${token}`,
		`Again this is an test ${token}`,
		`And one more an test ${token}`,
	].join('\n');

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);

	const rects = await page.evaluate(async (token) => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') return null;
		const annotated = await getAnnotatedText();
		const text = annotated.getText() as string;
		const needle = `an test ${token}`;
		const first = text.indexOf(needle);
		const second = text.indexOf(needle, first + 1);
		if (first < 0 || second < 0) return null;

		const getCaretRectAt = (idx: number) => {
			annotated.setSelection(idx, idx);
			const caret = document.querySelector('.kix-cursor-caret');
			const rect = caret?.getBoundingClientRect();
			if (!rect) return null;
			return { x: rect.x, y: rect.y };
		};

		return { first: getCaretRectAt(first), second: getCaretRectAt(second) };
	}, token);

	expect(rects).not.toBeNull();
	expect(rects?.first).not.toBeNull();
	expect(rects?.second).not.toBeNull();
	expect((rects?.second?.y ?? 0) - (rects?.first?.y ?? 0)).toBeGreaterThan(18);
});

test('Google Docs: Harper can write a suggestion on second line', async ({ page }) => {
	const token = `harper-gdocs-write-line2-${Date.now()}`;
	const input = [`First line clean ${token}`, `This is an test ${token}`].join('\n');
	const correctedNeedle = `This is a test ${token}`;

	await openGoogleDoc(page);
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
		.toContain(correctedNeedle);
});

test('Google Docs: applies expected suggestions across paragraph breaks', async ({ page }) => {
	test.setTimeout(120000);

	const input = `The Importance of Learning From Mistakes

Everyone makes mistake’s in there life, but not everyone understand how important they really are. When a person fails at something, they often feel embarrased or disapointed, and they might try to avoid thinking about it again. However, mistakes is actually one of the most powerful ways to learn, because they show us what doesnt work and force us too try different approches.
For example, a student who studies for a test but still gets a bad grade might think they are just not smart enough. But in reality, the problem could of been there study methods, or maybe they didnt focus on the right material. If they reflect on what went wrong, they can improve next time and do better. Without that failure, they wouldnt of had the motivation too change anything.
Another reason mistakes are valuable is because they build resilience. When people never experience difficulty, they become fragile and unprepared for real challenges. Struggling teaches patience, persistance, and problem solving skill’s that success alone can not provide. In fact, many famous inventors failed hundreds of time’s before they created something useful, which proves that failure is often part of success rather then the opposite.
In conclusion, mistakes should not be feared but embrased. They are essential for growth, learning, and developing strength of character. Instead of feeling ashamed, people should see errors as opportunity’s to become better, wiser, and more capable in the future.`;

	const applyCharEdit = async (
		needle: string,
		offsetStart: number,
		offsetEnd: number,
		replacement: string,
		expectedSnippet: string,
	) => {
		await expect
			.poll(async () => {
				const text = (await getGoogleDocText(page)) ?? '';
				return text.indexOf(needle);
			})
			.toBeGreaterThanOrEqual(0);

		const currentText = (await getGoogleDocText(page)) ?? '';
		const needleStart = currentText.indexOf(needle);
		expect(needleStart).toBeGreaterThanOrEqual(0);
		await replaceGoogleDocRange(
			page,
			needleStart + offsetStart,
			needleStart + offsetEnd,
			replacement,
		);

		await expect
			.poll(async () => normalizeGoogleDocText((await getGoogleDocText(page)) ?? ''), {
				timeout: 20000,
			})
			.toContain(expectedSnippet);
	};

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 20000 })
		.toBeGreaterThan(0);
	const highlightsBefore = await page.locator('#harper-highlight').count();
	await expect
		.poll(async () => normalizeGoogleDocText((await getGoogleDocText(page)) ?? ''), {
			timeout: 20000,
		})
		.toContain('The Importance of Learning From Mistakes');
	const headingLine = normalizeGoogleDocText((await getGoogleDocText(page)) ?? '').split('\n')[0] ?? '';
	expect(headingLine).toBe('The Importance of Learning From Mistakes');
	expect(headingLine.endsWith('.')).toBe(false);

	const textBefore = normalizeGoogleDocText((await getGoogleDocText(page)) ?? '');
	const paragraphCountBefore = textBefore.split('\n').length;

	// "too try" -> "to try" (delete one extra "o")
	await applyCharEdit('too try', 2, 3, '', 'force us to try');
	// "then" -> "than" (replace "e" with "a")
	await applyCharEdit('rather then the opposite', 9, 10, 'a', 'rather than the opposite');
	// "embrased" -> "embraced" (replace "s" with "c")
	await applyCharEdit('embrased', 5, 6, 'c', 'but embraced. They are essential');

	const finalText = normalizeGoogleDocText((await getGoogleDocText(page)) ?? '');
	expect(finalText.split('\n').length).toBe(paragraphCountBefore);
	expect(finalText).toContain('force us to try');
	expect(finalText).toContain('rather than the opposite');
	expect(finalText).toContain('but embraced. They are essential');

	const highlightsAfter = await page.locator('#harper-highlight').count();
	expect(highlightsAfter).toBeLessThanOrEqual(highlightsBefore);
});

test('Google Docs: bridge returns lower Y rect for lower-line lint', async ({ page }) => {
	const token = `harper-gdocs-bridge-rects-${Date.now()}`;
	const input = [
		`This is an test ${token}`,
		`Clean line ${token}`,
		`This is an test ${token} line three`,
	].join('\n');

	await openGoogleDoc(page);
	await replaceDocumentContent(page, input);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const rects = await page.evaluate(async (token) => {
		const getAnnotatedText = (window as any)._docs_annotate_getAnnotatedText;
		if (typeof getAnnotatedText !== 'function') return null;
		const annotated = await getAnnotatedText();
		const text = annotated.getText() as string;
		const firstNeedle = `an test ${token}`;
		const first = text.indexOf(firstNeedle);
		const second = text.indexOf(firstNeedle, first + 1);
		if (first < 0 || second < 0) return null;

		const readRects = (start: number, end: number, requestId: string) => {
			const bridge = document.getElementById('harper-google-docs-main-world-bridge');
			if (!bridge) return [];
			const attrName = `data-harper-rects-${requestId}`;
			document.dispatchEvent(
				new CustomEvent('harper:gdocs:get-rects', {
					detail: { requestId, start, end },
				}),
			);
			const raw = bridge.getAttribute(attrName);
			bridge.removeAttribute(attrName);
			if (!raw) return [];
			try {
				const parsed = JSON.parse(raw);
				return Array.isArray(parsed) ? parsed : [];
			} catch {
				return [];
			}
		};

		const firstRects = readRects(first, first + 7, `r1-${Date.now()}`);
		const secondRects = readRects(second, second + 7, `r2-${Date.now()}`);
		return { firstRects, secondRects };
	}, token);

	expect(rects).not.toBeNull();
	expect((rects?.firstRects?.length ?? 0) > 0).toBeTruthy();
	expect((rects?.secondRects?.length ?? 0) > 0).toBeTruthy();
	const firstY = rects?.firstRects?.[0]?.y ?? 0;
	const secondY = rects?.secondRects?.[0]?.y ?? 0;
	expect(secondY).toBeGreaterThan(firstY + 10);
});

test('Google Docs: highlight host remains non-interactive and top-layered', async ({ page }) => {
	const token = `harper-gdocs-host-style-${Date.now()}`;
	await openGoogleDoc(page);
	await replaceDocumentContent(page, `This is an test ${token}`);

	await expect
		.poll(async () => page.locator('#harper-highlight').count(), { timeout: 15000 })
		.toBeGreaterThan(0);

	const hostStyle = await page.evaluate(() => {
		const host = document.querySelector('#harper-highlight-host') as HTMLElement | null;
		if (!host) return null;
		const style = window.getComputedStyle(host);
		return {
			pointerEvents: style.pointerEvents,
			position: style.position,
			zIndex: style.zIndex,
		};
	});

	expect(hostStyle).not.toBeNull();
	expect(hostStyle?.pointerEvents).toBe('none');
	expect(hostStyle?.position).toBe('absolute');
	expect(Number(hostStyle?.zIndex ?? 0)).toBeGreaterThan(1000000);
});

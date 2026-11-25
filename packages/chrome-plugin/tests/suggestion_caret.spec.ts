import type { Locator } from '@playwright/test';
import { expect, test } from './fixtures';
import { clickHarperHighlight, getHarperHighlights, replaceEditorContent } from './testUtils';

async function caretOffset(editor: Locator) {
	return editor.evaluate((el) => {
		if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
			return el.selectionStart === el.selectionEnd ? el.selectionStart : null;
		}
		const selection = el.ownerDocument.getSelection();
		if (!selection?.isCollapsed || !el.contains(selection.focusNode)) return null;
		const beforeCaret = el.ownerDocument.createRange();
		beforeCaret.selectNodeContents(el);
		beforeCaret.setEnd(selection.focusNode!, selection.focusOffset);
		return beforeCaret.toString().length;
	});
}

const editors = [
	{
		name: 'GitHub textarea',
		page: 'github_textarea',
		selector: 'textarea',
		text: 'This is an test.',
		corrected: 'This is a test.',
		before: 'This is an test',
	},
	{
		name: 'editable body',
		page: 'editable_body',
		selector: 'body',
		text: 'This is an test of the Harper grammar checker.',
		corrected: 'This is a test of the Harper grammar checker.',
		before: 'This is an test of the Harper grammar checker',
	},
	{
		name: 'ProseMirror',
		page: 'prosemirror',
		selector: '.ProseMirror',
		text: 'This is an test.',
		corrected: 'This is a test.',
		before: 'This is an test',
	},
	{
		name: 'Lexical',
		page: 'lexical',
		selector: '[data-lexical-editor="true"]',
		text: 'This is an test.',
		corrected: 'This is a test.',
		before: 'This is an test',
	},
	{
		name: 'CodeMirror',
		page: 'codemirror',
		selector: '.cm-content',
		text: 'This is an test.',
		corrected: 'This is a test.',
		before: 'This is an test',
	},
	{
		name: 'Draft.js',
		page: 'draft',
		selector: '#rich-example .public-DraftEditor-content',
		text: 'This is an test.',
		corrected: 'This is a test.',
		before: 'This is an test',
	},
	{
		name: 'Slate',
		page: 'slate',
		selector: '[data-slate-editor="true"]',
		text: 'This is an test.',
		corrected: 'This is a test.',
		before: 'This is an test',
	},
];

for (const fixture of editors) {
	test(`Ctrl+E keeps the caret after a length-changing edit in ${fixture.name}`, async ({
		page,
	}) => {
		test.slow();
		await page.goto(`http://localhost:8081/${fixture.page}.html`);
		const editor = page.locator(fixture.selector);
		if (fixture.page === 'draft') await editor.click();
		if (fixture.page !== 'editable_body') await replaceEditorContent(editor, fixture.text);
		await expect(
			getHarperHighlights(page).and(page.locator('[data-problem-text="an"]')).first(),
		).toBeVisible({ timeout: 30000 });
		await editor.press('End');
		await page.keyboard.press('ArrowLeft');
		await expect.poll(() => caretOffset(editor)).toBe(fixture.before.length);

		await page.keyboard.press('Control+e');
		const readText = () =>
			editor.evaluate((el) => {
				if (el instanceof HTMLTextAreaElement) return el.value;
				return (el as HTMLElement).innerText.trim();
			});
		await expect.poll(readText).toBe(fixture.corrected);
		await expect(editor).toBeFocused();
		await expect.poll(() => caretOffset(editor)).toBe(fixture.before.length - 1);
		await page.keyboard.insertText(' again');
		await expect.poll(readText).toBe(`${fixture.corrected.slice(0, -1)} again.`);
	});
}

test('Ctrl+E keeps the caret before a length-changing edit', async ({ page }) => {
	await page.goto('http://localhost:8081/simple_textarea.html');
	const editor = page.locator('textarea');
	await editor.fill('This is an test.');
	await expect(getHarperHighlights(page)).toHaveCount(1);
	await editor.press('Home');
	await page.keyboard.press('ArrowRight');
	await expect.poll(() => caretOffset(editor)).toBe(1);
	await page.keyboard.press('Control+e');
	await expect(editor).toHaveValue('This is a test.');
	await expect.poll(() => caretOffset(editor)).toBe(1);
});

test('Ctrl+E keeps the caret with the same text when a correction adds a character', async ({
	page,
}) => {
	await page.goto('http://localhost:8081/simple_textarea.html');
	const editor = page.locator('textarea');
	await editor.fill('I saw a elephant.');
	await expect(
		getHarperHighlights(page).and(page.locator('[data-problem-text="a"]')),
	).toBeVisible();
	await editor.press('End');
	await page.keyboard.press('ArrowLeft');
	await expect.poll(() => caretOffset(editor)).toBe('I saw a elephant'.length);
	await page.keyboard.press('Control+e');
	await expect(editor).toHaveValue('I saw an elephant.');
	await expect.poll(() => caretOffset(editor)).toBe('I saw an elephant'.length);
});

test('Ctrl+E keeps the caret at the start of the next editable paragraph', async ({ page }) => {
	await page.goto('http://localhost:8081/editable_body.html');
	const editor = page.locator('body');
	await editor.evaluate((el) => {
		const first = el.querySelector('p')!;
		first.textContent = 'This is an test.';
		const next = el.ownerDocument.createElement('p');
		next.textContent = 'This paragraph stays unchanged.';
		el.replaceChildren(first, next);
		el.focus();
		el.ownerDocument.getSelection()!.collapse(next.firstChild, 0);
	});
	await expect(
		getHarperHighlights(page).and(page.locator('[data-problem-text="an"]')).first(),
	).toBeVisible();
	await expect.poll(() => caretOffset(editor)).toBe('This is an test.'.length);
	await page.keyboard.press('Control+e');
	await expect(editor.locator('p').first()).toHaveText('This is a test.');
	await expect.poll(() => caretOffset(editor)).toBe('This is a test.'.length);
	await page.keyboard.insertText('Next: ');
	await expect(editor.locator('p').last()).toHaveText('Next: This paragraph stays unchanged.');
});

test('Ctrl+E retains an offset within a word that is replaced', async ({ page }) => {
	await page.goto('http://localhost:8081/simple_textarea.html');
	const editor = page.locator('textarea');
	await editor.fill('This is a mistaek.');
	await expect(
		getHarperHighlights(page).and(page.locator('[data-problem-text="mistaek"]')),
	).toBeVisible();
	const offset = 'This is a mis'.length;
	await editor.evaluate((el, offset) => {
		(el as HTMLTextAreaElement).setSelectionRange(offset, offset);
	}, offset);
	await expect.poll(() => caretOffset(editor)).toBe(offset);
	await page.keyboard.press('Control+e');
	await expect(editor).toHaveValue('This is a mistake.');
	await expect.poll(() => caretOffset(editor)).toBe(offset);
});

test('Ctrl+E restores the caret after the editor commits its own replacement', async ({ page }) => {
	const url = 'http://localhost:8081/deferred_replacement.html';
	await page.route(url, (route) =>
		route.fulfill({
			contentType: 'text/html',
			body: '<div contenteditable="true" data-slate-editor="true">This is an test.</div>',
		}),
	);
	await page.goto(url);
	const editor = page.locator('[contenteditable]');
	await editor.evaluate((el) => {
		el.addEventListener('beforeinput', (event) => {
			if (!(event instanceof InputEvent) || event.inputType !== 'insertReplacementText') return;
			event.preventDefault();
			const edit = el.ownerDocument.getSelection()!.getRangeAt(0).cloneRange();
			el.addEventListener(
				'commit-replacement',
				() => {
					edit.deleteContents();
					edit.insertNode(el.ownerDocument.createTextNode(event.data!));
					el.ownerDocument.getSelection()!.collapse(edit.endContainer, edit.endOffset);
				},
				{ once: true },
			);
		});
	});
	await expect(getHarperHighlights(page)).toHaveCount(1);
	await editor.press('End');
	await page.keyboard.press('ArrowLeft');
	await expect.poll(() => caretOffset(editor)).toBe(15);
	await page.keyboard.press('Control+e');
	await expect(editor).toHaveText('This is an test.');
	await editor.dispatchEvent('commit-replacement');
	await expect(editor).toHaveText('This is a test.');
	await expect.poll(() => caretOffset(editor)).toBe(14);
});

test('A popup click places the caret after the applied suggestion', async ({ page }) => {
	await page.goto('http://localhost:8081/simple_textarea.html');
	const editor = page.locator('textarea');
	await editor.fill('This is an test.');
	await expect(getHarperHighlights(page)).toHaveCount(1);
	expect(await clickHarperHighlight(page, 'an')).toBe(true);
	await page.getByTitle('Click to replace "an" with "a"').click();
	await expect(editor).toHaveValue('This is a test.');
	await expect(editor).toBeFocused();
	await expect.poll(() => caretOffset(editor)).toBe('This is a'.length);
});

for (const behavior of [
	'never commits',
	'sets selection on the next frame',
	'sets selection just before the next key',
	'rebuilds an empty paragraph',
] as const) {
	test(`Ctrl+E preserves the caret when the editor ${behavior}`, async ({ page }) => {
		const url = 'http://localhost:8081/editor_owned_replacement.html';
		const rebuilt = behavior === 'rebuilds an empty paragraph';
		const tail = rebuilt ? '<p><br></p><p>More text.</p>' : '';
		await page.route(url, (route) =>
			route.fulfill({
				contentType: 'text/html',
				body: `<div contenteditable="true" data-slate-editor="true"><p>This is an test.</p>${tail}</div>`,
			}),
		);
		await page.goto(url);
		const editor = page.locator('[contenteditable]');
		await editor.evaluate(
			(el, { rebuilt, tail, behavior }) => {
				const selection = el.ownerDocument.getSelection()!;
				el.addEventListener('beforeinput', (event) => {
					if (!(event instanceof InputEvent) || event.inputType !== 'insertReplacementText') return;
					event.preventDefault();
					el.setAttribute('data-handled', 'true');
					if (behavior === 'never commits') return;
					el.addEventListener(
						'commit-replacement',
						() => {
							el.innerHTML = `<p>This is a test.</p>${tail}`;
							const moveSelection = () => {
								selection.collapse(el.querySelector('p')!.firstChild, 9);
								el.setAttribute('data-selection-written', 'true');
								if (behavior === 'sets selection just before the next key') {
									// Force keydown and its editing action before Harper's next frame.
									el.dispatchEvent(new KeyboardEvent('keydown', { key: 'x', bubbles: true }));
									el.ownerDocument.execCommand('insertText', false, 'x');
								}
							};
							if (rebuilt) moveSelection();
							else requestAnimationFrame(moveSelection);
						},
						{ once: true },
					);
				});
				(el as HTMLElement).focus();
				if (rebuilt) selection.collapse(el.querySelectorAll('p')[1], 0);
				else selection.collapse(el.querySelector('p')!.firstChild, 15);
			},
			{ rebuilt, tail, behavior },
		);
		await expect(
			getHarperHighlights(page).and(page.locator('[data-problem-text="an"]')).first(),
		).toBeVisible();
		await expect.poll(() => caretOffset(editor)).toBe(rebuilt ? 16 : 15);
		await page.keyboard.press('Control+e');
		await expect(editor).toHaveAttribute('data-handled', 'true');
		if (behavior === 'never commits') {
			await page.keyboard.press('x');
			await expect(editor.locator('p')).toHaveText('This is an testx.');
			return;
		}
		await editor.dispatchEvent('commit-replacement');
		await expect(editor).toHaveAttribute('data-selection-written', 'true');
		if (behavior === 'sets selection just before the next key') {
			await expect(editor.locator('p')).toHaveText('This is a testx.');
		} else if (rebuilt) {
			await expect
				.poll(() =>
					editor.evaluate((el) => {
						const selected = el.ownerDocument.getSelection();
						return (
							selected?.focusNode === el.querySelectorAll('p')[1] && selected?.focusOffset === 0
						);
					}),
				)
				.toBe(true);
			await page.keyboard.press('x');
			await expect(editor.locator('p')).toHaveText(['This is a test.', 'x', 'More text.']);
		} else {
			await expect.poll(() => caretOffset(editor)).toBe(14);
			await page.keyboard.press('x');
			await expect(editor.locator('p')).toHaveText('This is a testx.');
		}
	});
}

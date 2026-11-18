import { expect, test } from '@playwright/test';
import { replaceEditorContent } from './testUtils';

for (const kind of ['input', 'textarea', 'div', 'Draft.js']) {
	const tag = kind === 'Draft.js' ? 'div' : kind;
	const rich = tag === 'div';
	const attributes = rich
		? ` contenteditable="true"${kind === 'Draft.js' ? ' class="public-DraftEditor-content"' : ''}`
		: '';
	const method =
		kind === 'Draft.js'
			? 'uses character input for Draft.js'
			: rich
				? 'uses keyboard selection and bulk input for div'
				: `fills ${tag}`;

	test(`replaceEditorContent ${method}`, async ({ page }) => {
		await page.setContent(`<${tag}${attributes}></${tag}><output></output>`);
		await page.locator(tag).fill('Old text');
		await page.evaluate(() => {
			const values: string[] = [];
			document.addEventListener('input', (event) => {
				const target = event.target as HTMLElement;
				values.push('value' in target ? String(target.value) : (target.textContent ?? ''));
				document.querySelector('output')!.textContent = JSON.stringify(values);
			});
		});

		const text = 'This is a mistaek.';
		await replaceEditorContent(page.locator(tag), text);
		if (rich) {
			await expect(page.locator(tag)).toHaveText(text);
		} else {
			await expect(page.locator(tag)).toHaveValue(text);
		}
		// Native setup must not expose partial words. Draft.js is the legacy exception
		// because its selection model needs individual character key events.
		const values =
			kind === 'Draft.js'
				? ['', ...Array.from(text, (_, index) => text.slice(0, index + 1))]
				: rich
					? ['', text]
					: [text];
		await expect(page.locator('output')).toHaveText(JSON.stringify(values));
	});
}

test('replaceEditorContent preserves textarea line breaks', async ({ page }) => {
	await page.setContent('<textarea></textarea>');
	const editor = page.locator('textarea');
	await replaceEditorContent(editor, 'Valid words\ntset here.', true);
	await expect(editor).toHaveValue('Valid words\ntset here.');
});

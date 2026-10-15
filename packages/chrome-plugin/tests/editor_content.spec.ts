import { expect, test } from '@playwright/test';
import { replaceEditorContent } from './testUtils';

for (const tag of ['input', 'textarea', 'div']) {
	test(`replaceEditorContent uses keyboard input for ${tag}`, async ({ page }) => {
		const attributes = tag === 'div' ? ' contenteditable="true"' : '';
		await page.setContent(`<${tag}${attributes}></${tag}><output></output>`);
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
		if (tag === 'div') {
			await expect(page.locator(tag)).toHaveText(text);
		} else {
			await expect(page.locator(tag)).toHaveValue(text);
		}
		// The prefix "This is a m" has a different lint from the completed sentence.
		const prefixes = Array.from(text, (_, index) => text.slice(0, index + 1));
		await expect(page.locator('output')).toHaveText(JSON.stringify(prefixes));
	});
}

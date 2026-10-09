import { expect, test } from '@playwright/test';
import { replaceEditorContent } from './testUtils';

for (const tag of ['input', 'textarea']) {
	test(`Ensure \`replaceEditorContent\` fills \`${tag}\` without allowing intermediate text to be linted.`, async ({
		page,
	}) => {
		await page.setContent(`<${tag}></${tag}><output></output>`);
		await page.evaluate(() => {
			const values: string[] = [];
			document.addEventListener('input', (event) => {
				values.push((event.target as HTMLInputElement).value);
				document.querySelector('output')!.textContent = JSON.stringify(values);
			});
		});

		const text = 'This is a mistaek.';
		await replaceEditorContent(page.locator(tag), text);
		await expect(page.locator(tag)).toHaveValue(text);
		// The prefix "This is a m" has a different lint from the completed sentence.
		await expect(page.locator('output')).toHaveText(JSON.stringify([text]));
	});
}

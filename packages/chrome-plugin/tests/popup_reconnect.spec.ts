import { test, expect } from './fixtures';
import { clickHarperHighlight, getTextarea, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/popup_reconnect.html';

test('Reconnects the popup host before opening the popover', async ({ page }) => {
	const pageErrors: string[] = [];
	const consoleErrors: string[] = [];

	page.on('pageerror', (error) => {
		pageErrors.push(error.message);
	});
	page.on('console', (message) => {
		if (message.type() === 'error') {
			consoleErrors.push(message.text());
		}
	});

	await page.goto(TEST_PAGE_URL);

	const editor = getTextarea(page);
	await replaceEditorContent(editor, 'This is an test');

	await expect(page.locator('#harper-highlight').first()).toBeVisible();

	await page.evaluate(() => {
		(window as typeof window & { rehomeBodyPreservingApp?: () => void }).rehomeBodyPreservingApp?.();
	});

	await expect(page.locator('#harper-highlight').first()).toBeVisible();

	const opened = await clickHarperHighlight(page);
	expect(opened).toBe(true);
	await expect(page.locator('.harper-container')).toBeVisible();

	const errors = [...pageErrors, ...consoleErrors].join('\n');
	expect(errors).not.toContain("Failed to execute 'showPopover'");
	expect(errors).not.toContain('Invalid on disconnected popover elements');
});

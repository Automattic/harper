import { expect, test } from './fixtures';
import {
	getTextarea,
	openHarperPopupFromEditorPointerDown,
	replaceEditorContent,
	waitForHarperHighlightCenter,
} from './testUtils';

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

	expect(await waitForHarperHighlightCenter(page, 12000)).not.toBeNull();

	await page.evaluate(() => {
		(
			window as typeof window & {
				rehomeBodyPreservingApp?: () => void;
			}
		).rehomeBodyPreservingApp?.();
	});

	const opened = await openHarperPopupFromEditorPointerDown(page, editor);
	expect(opened).toBe(true);

	const errors = [...pageErrors, ...consoleErrors].join('\n');
	expect(errors).not.toContain("Failed to execute 'showPopover'");
	expect(errors).not.toContain('Invalid on disconnected popover elements');
});

import { expect, test } from './fixtures';
import {
	getTextarea,
	openHarperPopupFromEditorPointerDown,
	replaceEditorContent,
	waitForHighlightsToSettle,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/popup_reconnect.html';

test('Reconnects the popup host before opening the popover', async ({ page }) => {
	test.slow();
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

	// Finish linting before disconnecting the popup host. Do not wait for a render
	// after replacing the body: the pointer handler must reconnect the host itself.
	await waitForHighlightsToSettle(page);

	await page.evaluate(() => {
		(
			window as typeof window & {
				rehomeBodyPreservingApp?: () => void;
			}
		).rehomeBodyPreservingApp?.();
	});

	const opened = await openHarperPopupFromEditorPointerDown(page, editor);
	expect(opened).toBe(true);
	await expect(page.getByTitle('Click to replace "an" with "a"')).toBeVisible();

	const errors = [...pageErrors, ...consoleErrors].join('\n');
	expect(errors).not.toContain("Failed to execute 'showPopover'");
	expect(errors).not.toContain('Invalid on disconnected popover elements');
});

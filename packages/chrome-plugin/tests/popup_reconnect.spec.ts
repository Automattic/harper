import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { getTextarea, replaceEditorContent } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/popup_reconnect.html';

async function waitForHighlightCenter(
	page: Page,
	timeoutMs = 12000,
): Promise<{ x: number; y: number } | null> {
	return await page.evaluate(async (timeoutMs) => {
		const findHighlightCenter = () => {
			const hosts = Array.from(document.querySelectorAll('harper-render-box')) as HTMLElement[];
			for (const host of hosts) {
				const highlight = host.shadowRoot?.querySelector('#harper-highlight');
				if (!(highlight instanceof HTMLElement)) {
					continue;
				}

				const rect = highlight.getBoundingClientRect();
				return {
					x: rect.left + rect.width / 2,
					y: rect.top + rect.height / 2,
				};
			}

			return null;
		};

		const deadline = performance.now() + timeoutMs;

		while (performance.now() < deadline) {
			const center = findHighlightCenter();
			if (center != null) {
				return center;
			}

			await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
		}

		return null;
	}, timeoutMs);
}

async function triggerPopupFromHighlightPosition(page: Page): Promise<boolean> {
	const editor = page.locator('textarea');
	if ((await editor.count()) === 0) {
		return false;
	}

	const center = await waitForHighlightCenter(page, 12000);
	if (center == null) {
		return false;
	}

	return await page.evaluate(async ({ x, y }) => {
		const editor = document.querySelector('textarea');
		if (!(editor instanceof HTMLElement)) {
			return false;
		}

		const isPopupVisible = () => {
			const hosts = Array.from(document.querySelectorAll('harper-render-box')) as HTMLElement[];
			return hosts.some((host) => {
				const container = host.shadowRoot?.querySelector('.harper-container');
				return (
					container instanceof HTMLElement &&
					host.isConnected &&
					host.matches(':popover-open') &&
					getComputedStyle(host).visibility === 'visible'
				);
			});
		};

		const center = { x, y };
		editor.dispatchEvent(
			new PointerEvent('pointerdown', {
				bubbles: true,
				composed: true,
				button: 0,
				buttons: 1,
				clientX: center.x,
				clientY: center.y,
				pointerId: 1,
				pointerType: 'mouse',
				screenX: center.x,
				screenY: center.y,
			}),
		);

		const deadline = performance.now() + 2000;
		while (performance.now() < deadline) {
			if (isPopupVisible()) {
				return true;
			}

			await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
		}

		return false;
	}, center);
}

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

	expect(await waitForHighlightCenter(page, 12000)).not.toBeNull();

	await page.evaluate(() => {
		(
			window as typeof window & {
				rehomeBodyPreservingApp?: () => void;
			}
		).rehomeBodyPreservingApp?.();
	});

	const opened = await triggerPopupFromHighlightPosition(page);
	expect(opened).toBe(true);

	const errors = [...pageErrors, ...consoleErrors].join('\n');
	expect(errors).not.toContain("Failed to execute 'showPopover'");
	expect(errors).not.toContain('Invalid on disconnected popover elements');
});

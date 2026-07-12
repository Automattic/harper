import type { BrowserContext, Page } from '@playwright/test';
import path from 'path';
import { createFixture } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');
const { test, expect } = createFixture(pathToExtension);

async function getBackgroundForCleanup(context: BrowserContext) {
	return (
		context.serviceWorkers()[0] ??
		context.backgroundPages()[0] ??
		(await Promise.race([
			context.waitForEvent('serviceworker', { timeout: 5000 }).catch(() => null),
			context.waitForEvent('backgroundpage', { timeout: 5000 }).catch(() => null),
		]))
	);
}

/** Get the background page for a context, used to access extension internals. */
async function getBackground(context: BrowserContext): Promise<any> {
	return (
		context.serviceWorkers()[0] ??
		context.backgroundPages()[0] ??
		(await Promise.race([
			context.waitForEvent('serviceworker', { timeout: 90000 }).catch(() => null),
			context.waitForEvent('backgroundpage', { timeout: 90000 }).catch(() => null),
		]))
	);
}

// Ensure tests run with a consistent dialect (American English) for predictable results
test.beforeEach(async ({ context }) => {
	const bg = await getBackground(context);
	if (bg) {
		await bg.evaluate(
			async () => {
				// Ensure the linter is initialized with American dialect for consistent test behavior
				// This prevents locale detection from affecting test results
				// Dialect enum: American = 0, British = 1, Australian = 2, Canadian = 3, Indian = 4
				await chrome.storage.local.set({ dialect: 0 });
			},
		);
	}
});

test.afterEach(async ({ context }) => {
	const bg = await getBackgroundForCleanup(context);
	if (bg) {
		await bg.evaluate(
			() =>
				new Promise<void>((resolve) => {
					chrome.storage.local.clear(resolve);
				}),
		);
	}
});

export { test, expect };
export { getBackground };

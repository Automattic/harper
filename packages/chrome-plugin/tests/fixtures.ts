import type { BrowserContext } from '@playwright/test';
import path from 'path';
import { createFixture } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');
const { test: base, expect } = createFixture(pathToExtension);

const test = base.extend({
	browser: [
		async ({ browser }, use) => {
			await use(browser);
			if (browser.isConnected()) {
				await browser.close();
			}
		},
		{ scope: 'worker', timeout: 0 },
	],
	context: async ({ browserName, context }, use) => {
		await use(context);
		if (browserName === 'chromium') {
			await context.close();
		}
	},
});

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

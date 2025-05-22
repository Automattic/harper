import path from 'path';
import { type BrowserContext, test as base, chromium } from '@playwright/test';

export const test = base.extend<{
	context: BrowserContext;
	extensionId: string;
}>({
	// biome-ignore lint/correctness/noEmptyPattern: it's by Playwright. Explanation not provided.
	context: async ({}, use) => {
		const pathToExtension = path.join(import.meta.dirname, '../build');
		console.log(`Loading extension from ${pathToExtension}`);
		const context = await chromium.launchPersistentContext('', {
			channel: 'chromium',
			args: [
				`--disable-extensions-except=${pathToExtension}`,
				`--load-extension=${pathToExtension}`,
			],
		});
		await use(context);
		await context.close();
	},
	extensionId: async ({ context }, use) => {
		let [background] = context.serviceWorkers();
		if (!background) background = await context.waitForEvent('serviceworker');

		const extensionId = background.url().split('/')[2];
		await use(extensionId);
	},
});

export const expect = test.expect;

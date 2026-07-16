import { mkdir, rm } from 'node:fs/promises';
import type { BrowserContext } from '@playwright/test';
import path from 'path';
import { test as base, expect } from '@playwright/test';
import { withExtension } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');

const test = base.extend({
	context: async (
		{ playwright, browserName, contextOptions, launchOptions, headless },
		use,
		testInfo,
	) => {
		if (browserName === 'chromium' && headless) {
			throw new Error('Chromium extensions require headed mode');
		}

		const profile = testInfo.outputPath('browser-profile');
		await mkdir(profile, { recursive: true });

		const browserType = withExtension(playwright[browserName], pathToExtension);
		const context = await browserType.launchPersistentContext(profile, {
			...contextOptions,
			...launchOptions,
			headless,
		});

		try {
			await use(context);
		} finally {
			await context.close();
			await rm(profile, { recursive: true, force: true });
		}
	},
});

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
				// German variants start at higher values
				await chrome.storage.local.set({ dialect: 0 });
			},
		);
		// Wait for the storage change to propagate and linter to reinitialize
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
});

export { test, expect };
export { getBackground };

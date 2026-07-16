import { mkdir, rm } from 'node:fs/promises';
import path from 'node:path';
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

export { test, expect };
export { getBackground };

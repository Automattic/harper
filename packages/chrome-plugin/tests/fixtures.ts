import { mkdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { test as base, expect } from '@playwright/test';
import { withExtension } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');
const welcomeUrl = 'https://writewithharper.com/install-browser-extension';

const test = base.extend({
	context: [
		async ({ playwright, browserName, contextOptions, launchOptions, headless }, use, testInfo) => {
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
				// The install tab can steal focus and pause Firefox's animation frames.
				// Stub its network load, then close it before creating the test page.
				await context.route(`${welcomeUrl}**`, (route) =>
					route.fulfill({ contentType: 'text/html', body: '<title>Installed</title>' }),
				);
				await expect
					.poll(() => context.pages().some((page) => page.url().startsWith(welcomeUrl)), {
						timeout: 90000,
						message: 'The extension install hook should open its welcome tab',
					})
					.toBe(true);
				for (const page of context.pages()) {
					if (page.url().startsWith(welcomeUrl)) await page.close();
				}

				// A completed lint proves that the background linter and content script
				// are ready. Keep cold startup out of each test's UI assertion budget.
				// A normal page works in Firefox too, which does not expose background pages.
				const startupPage = await context.newPage();
				try {
					await startupPage.goto('http://localhost:8081/simple_textarea.html');
					await startupPage.bringToFront();
					await startupPage.locator('textarea').fill('This is an test');
					await expect(startupPage.locator('#harper-highlight').first()).toBeVisible({
						timeout: 90000,
					});
				} finally {
					await startupPage.close();
				}

				await use(context);
			} finally {
				await context.close();
				await rm(profile, { recursive: true, force: true });
			}
		},
		{ scope: 'test', timeout: 120000 },
	],
});

export { test, expect };

import { defineConfig, devices } from '@playwright/test';

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
	testDir: './tests',
	testIgnore: ['**/google_docs*.spec.ts', '**/googleDocs*.spec.ts'],
	fullyParallel: true,
	/* Fail the build on CI if you accidentally left test.only in the source code. */
	forbidOnly: !!process.env.CI,
	/* Retry on CI only */
	retries: process.env.CI ? 4 : 0,
	/* Extension tests share one browser extension background; keep storage teardown isolated. */
	workers: 1,
	/* Reporter to use. See https://playwright.dev/docs/test-reporters */
	reporter: 'html',
	use: {
		/* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
		trace: 'on-first-retry',
	},
	/** A half hour */
	globalTimeout: 1800000,
	webServer: {
		command: 'pnpm exec http-server ./tests/pages -p 8081 -a 127.0.0.1',
		url: 'http://127.0.0.1:8081',
		reuseExistingServer: true,
		stdout: 'pipe',
		stderr: 'pipe',
	},
	/* Configure projects for major browsers */
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] },
		},
		{
			name: 'firefox',
			use: { ...devices['Desktop Firefox'] },
		},
	],
});

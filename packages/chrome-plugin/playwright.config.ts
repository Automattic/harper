import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, devices } from '@playwright/test';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const wrapperPath = path.resolve(__dirname, './vglrunWrapper.js');

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
	testDir: './tests',
	testIgnore: ['**/google_docs*.spec.ts', '**/googleDocs*.spec.ts'],
	fullyParallel: true,
	/* Fail the build on CI if you accidentally left test.only in the source code. */
	forbidOnly: !!process.env.CI,
	/** Extremely important to avoid flaky tests. DO NOT CHANGE or I will kill you. */
	retries: 0,
	/** Extremely important to avoid flaky tests. DO NOT CHANGE or I will kill you. */
	repeatEach: 3,
	/* Reporter to use. See https://playwright.dev/docs/test-reporters */
	reporter: 'html',
	use: {
		/* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
		trace: 'on-first-retry',
		screenshot: 'only-on-failure',
	},
	/** One hour */
	globalTimeout: 3600000,
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
			workers: process.env.CI ? 1 : '50%',
			use: {
				...devices['Desktop Chrome'],
				launchOptions: {
					executablePath: wrapperPath,
					args: [
						'--disable-gpu-sandbox',
						'--use-gl=desktop',
						'--use-angle=vulkan',
						'--enable-features=Vulkan',
						'--disable-vulkan-surface',
						'--enable-unsafe-webgpu',
					],
				},
			},
		},
		{
			name: 'firefox',
			workers: process.env.CI ? 1 : '50%',
			use: { ...devices['Desktop Firefox'] },
		},
	],
});

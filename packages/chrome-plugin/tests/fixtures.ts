import path from 'path';
import { createFixture } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');
const initScriptPath = path.join(import.meta.dirname, './initScript.js');
const { test, expect } = createFixture(pathToExtension);

// Simulate browsers without the CSS Custom Highlight API so we exercise the fallback path.
test.beforeEach(async ({ page }) => {
	await page.addInitScript({ path: initScriptPath });
});

test.afterEach(async ({ context }) => {
	const bg = context.serviceWorkers()[0] ?? context.backgroundPages()[0];
	if (bg) await bg.evaluate(() => chrome?.storage?.local.clear?.());
});

export { test, expect };

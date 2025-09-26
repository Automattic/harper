import path from 'path';
import { createFixture } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');
const { test, expect } = createFixture(pathToExtension);

test.afterEach(async ({ context }) => {
	const bg = context.serviceWorkers()[0] ?? context.backgroundPages()[0];
	if (bg) await bg.evaluate(() => chrome?.storage?.local.clear?.());
});

export { test, expect };

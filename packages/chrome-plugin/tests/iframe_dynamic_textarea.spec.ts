import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { replaceEditorContent } from './testUtils';

const PARENT_DOMAIN = 'localhost';
const TEST_PAGE_URL = 'http://localhost:8081/iframe_dynamic_textarea_parent.html';

async function getBackground(context: BrowserContext) {
	return (
		context.serviceWorkers()[0] ??
		context.backgroundPages()[0] ??
		(await context.waitForEvent('serviceworker'))
	);
}

async function seedDomainSettings(
	context: BrowserContext,
	domains: Record<string, boolean>,
	defaultEnable = false,
) {
	const background = await getBackground(context);
	const storageEntries = Object.fromEntries(
		Object.entries(domains).map(([domain, enabled]) => [`domainStatus ${domain}`, enabled]),
	);

	await background.evaluate(
		async ({ defaultEnable, storageEntries }) => {
			await chrome.storage.local.set({ defaultEnable, ...storageEntries });
		},
		{ defaultEnable, storageEntries },
	);
}

function getChildFrame(page: Page) {
	return page.frameLocator('#editor-frame');
}

function getRevealButton(page: Page) {
	return getChildFrame(page).getByRole('button', { name: 'Enter manually' });
}

function getChildTextarea(page: Page) {
	return getChildFrame(page).locator('#editor');
}

function getChildHighlights(page: Page) {
	return getChildFrame(page).locator('#harper-highlight');
}

async function clickChildHighlight(page: Page) {
	const highlight = getChildHighlights(page).first();
	await highlight.waitFor({ state: 'visible', timeout: 12000 });

	const box = await highlight.boundingBox();
	if (box == null) {
		throw new Error('Expected iframe highlight to have a bounding box.');
	}

	await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
}

test('attaches to a dynamically revealed iframe textarea on focus', async ({ context, page }) => {
	test.slow();
	await seedDomainSettings(context, { [PARENT_DOMAIN]: true });

	await page.goto(TEST_PAGE_URL);
	await getRevealButton(page).click();

	const editor = getChildTextarea(page);
	await editor.waitFor({ state: 'visible', timeout: 5000 });
	await replaceEditorContent(editor, 'This is an test');

	await clickChildHighlight(page);
	await expect(getChildFrame(page).locator('.harper-container')).toBeVisible();
});

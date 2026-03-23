import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { replaceEditorContent } from './testUtils';

const PARENT_DOMAIN = 'parent.localhost';
const CHILD_DOMAIN = '127.0.0.1';
const TEST_PAGE_URL = 'http://parent.localhost:8081/iframe_parent_origin.html';
const TEST_PAGE_WWW_URL = 'http://www.parent.localhost:8081/iframe_parent_origin.html';

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

function getChildTextarea(page: Page) {
	return getChildFrame(page).locator('textarea');
}

function getChildHighlights(page: Page) {
	return getChildFrame(page).locator('#harper-highlight');
}

function getChildPopup(page: Page) {
	return getChildFrame(page).locator('.harper-container');
}

async function typeLintableText(page: Page) {
	await replaceEditorContent(getChildTextarea(page), 'This is an test');
}

async function waitForChildHighlight(page: Page) {
	const highlight = getChildHighlights(page).first();
	await highlight.waitFor({ state: 'visible', timeout: 12000 });
	return highlight;
}

async function clickChildHighlight(page: Page) {
	const highlight = await waitForChildHighlight(page);
	const box = await highlight.boundingBox();

	if (box == null) {
		throw new Error('Expected iframe highlight to have a bounding box.');
	}

	await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
}

async function expectNoChildHighlights(page: Page) {
	await page.waitForTimeout(3000);
	await expect(getChildHighlights(page)).toHaveCount(0);
}

test.describe('parent-origin inheritance', () => {
	test('inherits the parent enabled state when the iframe host is unset', async ({
		context,
		page,
	}) => {
		test.slow();
		await seedDomainSettings(context, { [PARENT_DOMAIN]: true });

		await page.goto(TEST_PAGE_URL);
		await typeLintableText(page);
		await clickChildHighlight(page);

		await expect(getChildPopup(page)).toBeVisible();
	});

	test('inherits when the parent is `www.` but the stored key follows the popup pattern', async ({
		context,
		page,
	}) => {
		test.slow();
		await seedDomainSettings(context, { [PARENT_DOMAIN]: true });

		await page.goto(TEST_PAGE_WWW_URL);
		await typeLintableText(page);
		await clickChildHighlight(page);

		await expect(getChildPopup(page)).toBeVisible();
	});

	test('preserves an explicit iframe opt-out when the parent is enabled', async ({
		context,
		page,
	}) => {
		await seedDomainSettings(context, {
			[PARENT_DOMAIN]: true,
			[CHILD_DOMAIN]: false,
		});

		await page.goto(TEST_PAGE_URL);
		await typeLintableText(page);

		await expectNoChildHighlights(page);
	});

	test('does not inherit when the parent is disabled and the iframe host is unset', async ({
		context,
		page,
	}) => {
		await seedDomainSettings(context, { [PARENT_DOMAIN]: false });

		await page.goto(TEST_PAGE_URL);
		await typeLintableText(page);

		await expectNoChildHighlights(page);
	});

	test('preserves an explicit iframe opt-in even when the parent is disabled', async ({
		context,
		page,
	}) => {
		test.slow();
		await seedDomainSettings(context, {
			[PARENT_DOMAIN]: false,
			[CHILD_DOMAIN]: true,
		});

		await page.goto(TEST_PAGE_URL);
		await typeLintableText(page);

		// This precedence is intentional for now because explicit user overrides should win.
		// Confirm it with maintainers if the project wants parent-level disablement to dominate.
		await clickChildHighlight(page);

		await expect(getChildPopup(page)).toBeVisible();
	});
});

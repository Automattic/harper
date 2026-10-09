import { readFile } from 'node:fs/promises';
import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { getBackground, openExtensionPage } from './testUtils';

const IMPORTED_WORD = 'qxjimporttest';

function importFilePayload() {
	return {
		name: 'harper-settings.json',
		mimeType: 'application/json',
		buffer: Buffer.from(
			JSON.stringify({
				schema_version: 1,
				exported_at: '2026-10-09T10:00:00.000Z',
				source_app: 'harper-firefox',
				harper_version: '0.0.0',
				core: {
					dialect: 'American',
					lint_config: {},
					user_dictionary: [IMPORTED_WORD],
					ignored_lints: '{"context_hashes":[]}',
				},
			}),
		),
	};
}

async function openBackupSection(context: BrowserContext, page: Page) {
	await getBackground(context);
	await openExtensionPage(context, page, 'options.html');
	await expect(page.getByText('Backup & Restore')).toBeVisible({ timeout: 15000 });
}

test.describe('options backup and restore', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('exports a valid settings file', async ({ context, page }, testInfo) => {
		await openBackupSection(context, page);

		const [download] = await Promise.all([
			page.waitForEvent('download'),
			page.getByRole('button', { name: 'Export', exact: true }).click(),
		]);
		const target = testInfo.outputPath('harper-settings.json');
		await download.saveAs(target);

		const parsed = JSON.parse(await readFile(target, 'utf-8'));
		expect(parsed.schema_version).toBe(1);
		expect(typeof parsed.core.dialect).toBe('string');
		expect(Array.isArray(parsed.core.user_dictionary)).toBe(true);
	});

	test('imports a file after confirmation', async ({ context, page }) => {
		await openBackupSection(context, page);

		await page.locator('input[type="file"]').setInputFiles([importFilePayload()]);
		await expect(page.getByRole('button', { name: 'Confirm import' })).toBeVisible();
		await expect(page.getByText('word(s) added')).toBeVisible();

		await page.getByRole('button', { name: 'Confirm import' }).click();
		await page.waitForLoadState('load');
		await expect(page.getByText('Settings imported.')).toBeVisible({ timeout: 15000 });

		const stored = await page.evaluate(() => chrome.storage.local.get('userDictionary'));
		expect(stored.userDictionary).toContain(IMPORTED_WORD);
	});

	test('shows an error for a bad file without navigating', async ({ context, page }) => {
		await openBackupSection(context, page);

		await page.locator('input[type="file"]').setInputFiles([
			{
				name: 'bad.json',
				mimeType: 'application/json',
				buffer: Buffer.from('{"schema_version": 99}'),
			},
		]);

		await expect(page.getByText(/schema_version/)).toBeVisible();
		await expect(page.getByRole('button', { name: 'Confirm import' })).toHaveCount(0);
	});
});
